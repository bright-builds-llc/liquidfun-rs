//! Black-box consumer evidence for restricted hooks and the step lifecycle.

use std::collections::VecDeque;
use std::panic::{AssertUnwindSafe, catch_unwind};

use liquidfun::{
    CollisionDirective, CommandError, ContactSnapshot, ContactView, HandleError, PreSolveDirective,
    StepError, StepHook, StepLimits, World, WorldCommand,
};

fn world_with_contact() -> (World, ContactSnapshot) {
    let mut world = World::new().expect("test world key should remain available");
    let body = world.create_body().expect("body should fit");
    let first = world.create_fixture(body).expect("fixture should fit");
    let second = world.create_fixture(body).expect("fixture should fit");
    (world, ContactSnapshot::new(first, second))
}

#[derive(Default)]
struct RecordingHook {
    observed: Vec<[liquidfun::FixtureId; 2]>,
}

impl StepHook for RecordingHook {
    fn pre_solve(&mut self, _contact: ContactView<'_>) -> PreSolveDirective {
        PreSolveDirective::Disable
    }

    fn observe(&mut self, contact: ContactView<'_>) {
        self.observed.push(contact.fixtures());
    }
}

#[test]
fn owned_events_preserve_hook_order_multiplicity_and_directives() {
    // Arrange
    let (mut world, contact) = world_with_contact();
    let contacts = [contact, contact];
    let mut hook = RecordingHook::default();

    // Act
    let report = world
        .step(&contacts, &mut hook, StepLimits::default())
        .expect("contacts should be valid and bounded");

    // Assert
    assert_eq!(report.events().len(), 2);
    assert_eq!(report.events()[0].contact(), contact);
    assert_eq!(report.events()[1].contact(), contact);
    assert_eq!(report.events()[0].collision(), CollisionDirective::Collide);
    assert_eq!(
        report.events()[0].maybe_pre_solve(),
        Some(PreSolveDirective::Disable)
    );
    assert_eq!(hook.observed, vec![contact.fixtures(), contact.fixtures()]);
}

struct CommandHook {
    commands: VecDeque<WorldCommand>,
}

impl StepHook for CommandHook {
    fn command(&mut self, _contact: ContactView<'_>) -> Option<WorldCommand> {
        self.commands.pop_front()
    }
}

#[test]
fn deferred_commands_apply_only_after_all_hook_dispatch_unlocks() {
    // Arrange
    let (mut world, contact) = world_with_contact();
    let contact_body = world.create_body().expect("body should fit");
    let first = world
        .create_fixture(contact_body)
        .expect("fixture should fit");
    let second = world
        .create_fixture(contact_body)
        .expect("fixture should fit");
    let command_contact = ContactSnapshot::new(first, second);
    let mut hook = CommandHook {
        commands: [WorldCommand::DestroyBody(contact_body)].into(),
    };

    // Act
    let report = world
        .step(
            &[command_contact, command_contact, contact],
            &mut hook,
            StepLimits::default(),
        )
        .expect("the first command must wait until every contact is dispatched");

    // Assert
    assert!(!world.is_locked());
    assert!(!world.contains_body(contact_body));
    assert_eq!(report.command_applications().len(), 1);
    assert!(report.command_applications()[0].result().is_ok());
}

#[test]
fn stale_command_result_does_not_hide_later_command_success() {
    // Arrange
    let (mut world, contact) = world_with_contact();
    let stale = world.create_body().expect("body should fit");
    world.destroy_body(stale).expect("body should be live");
    let live = world.create_body().expect("body should fit");
    let mut hook = CommandHook {
        commands: [
            WorldCommand::DestroyBody(stale),
            WorldCommand::DestroyBody(live),
        ]
        .into(),
    };

    // Act
    let report = world
        .step(&[contact, contact], &mut hook, StepLimits::default())
        .expect("invalid commands are reported per application");

    // Assert
    assert_eq!(
        report.command_applications()[0].result(),
        Err(CommandError::InvalidHandle(HandleError::StaleOrDestroyed))
    );
    assert!(report.command_applications()[1].result().is_ok());
    assert!(!world.contains_body(live));
}

#[test]
fn finite_event_limit_fails_without_applying_commands() {
    // Arrange
    let (mut world, contact) = world_with_contact();
    let body = world.create_body().expect("body should fit");
    let mut hook = CommandHook {
        commands: [WorldCommand::DestroyBody(body)].into(),
    };
    let limits = StepLimits::new(1, 1).expect("limits should be below hard maxima");

    // Act
    let result = world.step(&[contact, contact], &mut hook, limits);

    // Assert
    assert_eq!(
        result,
        Err(StepError::LimitExceeded {
            resource: "event",
            limit: 1,
        })
    );
    assert!(world.contains_body(body));
}

struct PanickingHook;

impl StepHook for PanickingHook {
    fn observe(&mut self, _contact: ContactView<'_>) {
        panic!("intentional consumer hook panic");
    }
}

#[test]
fn hook_panic_restores_lock_and_poison_gates_later_operations() {
    // Arrange
    let (mut world, contact) = world_with_contact();
    let body = world.create_body().expect("body should fit");
    let mut hook = PanickingHook;

    // Act
    let panic = catch_unwind(AssertUnwindSafe(|| {
        let _result = world.step(&[contact], &mut hook, StepLimits::default());
    }));

    // Assert
    assert!(panic.is_err());
    assert!(!world.is_locked());
    assert!(world.is_poisoned());
    assert_eq!(world.destroy_body(body), Err(HandleError::WorldPoisoned));
    assert_eq!(
        world.step(&[], &mut hook, StepLimits::default()),
        Err(StepError::Poisoned)
    );
}
