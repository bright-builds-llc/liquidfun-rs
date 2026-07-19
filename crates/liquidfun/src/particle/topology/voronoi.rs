use std::collections::VecDeque;

use crate::math::Vec2;

use super::{VoronoiError, VoronoiLimits};

#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) struct VoronoiGenerator {
    center: Vec2,
    necessary: bool,
}

impl VoronoiGenerator {
    pub(super) const fn new(center: Vec2, necessary: bool) -> Self {
        Self { center, necessary }
    }

    pub(super) const fn necessary(self) -> bool {
        self.necessary
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct VoronoiNode {
    generator_ordinals: [usize; 3],
}

impl VoronoiNode {
    pub(super) const fn generator_ordinals(self) -> [usize; 3] {
        self.generator_ordinals
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct VoronoiDiagram {
    width: usize,
    height: usize,
    owners: Box<[Option<usize>]>,
    nodes: Box<[VoronoiNode]>,
}

impl VoronoiDiagram {
    pub(super) fn generate(
        generators: &[VoronoiGenerator],
        radius: f32,
        margin: f32,
        limits: VoronoiLimits,
    ) -> Result<Self, VoronoiError> {
        validate_inputs(generators, radius, margin, limits)?;
        let Some((lower, upper)) = necessary_bounds(generators) else {
            return Ok(Self::empty());
        };
        let inverse_radius = 1.0 / radius;
        let grid = GridSpec::checked(
            lower,
            upper,
            inverse_radius,
            margin,
            generators.len(),
            limits,
        )?;
        let scaled_generators = scale_generators(generators, grid.lower, inverse_radius)?;
        let mut owners = allocated_owners(grid.cells)?;
        let mut queue = BoundedQueue::new(grid.queue_capacity)?;
        seed_generators(&scaled_generators, grid, &mut queue)?;
        let mut work = WorkCounter::new(limits.work);
        flood_fill(&mut owners, grid, &mut queue, &mut work)?;
        seed_relaxation(&owners, grid, &mut queue)?;
        relax_distances(&mut owners, &scaled_generators, grid, &mut queue, &mut work)?;
        let nodes = collect_nodes(&owners, &scaled_generators, grid, limits.nodes)?;
        Ok(Self {
            width: grid.width,
            height: grid.height,
            owners: owners.into_boxed_slice(),
            nodes: nodes.into_boxed_slice(),
        })
    }

    pub(super) fn nodes(&self) -> &[VoronoiNode] {
        &self.nodes
    }

    fn empty() -> Self {
        Self {
            width: 0,
            height: 0,
            owners: Box::new([]),
            nodes: Box::new([]),
        }
    }

    #[cfg(test)]
    fn owner_ordinal(&self, x: usize, y: usize) -> Option<usize> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.owners[x + y * self.width]
    }

    #[cfg(test)]
    const fn dimensions(&self) -> (usize, usize) {
        (self.width, self.height)
    }
}

#[derive(Debug, Clone, Copy)]
struct ScaledGenerator {
    center: Vec2,
    necessary: bool,
}

#[derive(Debug, Clone, Copy)]
struct GridSpec {
    lower: Vec2,
    width: usize,
    height: usize,
    cells: usize,
    queue_capacity: usize,
}

impl GridSpec {
    fn checked(
        mut lower: Vec2,
        mut upper: Vec2,
        inverse_radius: f32,
        margin: f32,
        generator_count: usize,
        limits: VoronoiLimits,
    ) -> Result<Self, VoronoiError> {
        lower.x -= margin;
        lower.y -= margin;
        upper.x += margin;
        upper.y += margin;
        if !lower.is_valid() || !upper.is_valid() {
            return Err(VoronoiError::NonFiniteDerivedGeometry);
        }
        let width = axis_count(lower.x, upper.x, inverse_radius)?;
        let height = axis_count(lower.y, upper.y, inverse_radius)?;
        let cells = width
            .checked_mul(height)
            .ok_or(VoronoiError::ArithmeticOverflow)?;
        check_limit(cells, limits.cells, |required, limit| {
            VoronoiError::GridLimitExceeded { required, limit }
        })?;
        let source_queue_capacity = cells
            .checked_mul(4)
            .ok_or(VoronoiError::ArithmeticOverflow)?;
        let queue_capacity = source_queue_capacity.max(generator_count);
        check_limit(queue_capacity, limits.queue_tasks, |required, limit| {
            VoronoiError::QueueLimitExceeded { required, limit }
        })?;
        let possible_nodes = width
            .saturating_sub(1)
            .checked_mul(height.saturating_sub(1))
            .and_then(|count| count.checked_mul(2))
            .ok_or(VoronoiError::ArithmeticOverflow)?;
        check_limit(possible_nodes, limits.nodes, |required, limit| {
            VoronoiError::NodeLimitExceeded { required, limit }
        })?;
        let work_bound = work_bound(cells, generator_count, width, height)?;
        check_limit(work_bound, limits.work, |required, limit| {
            VoronoiError::WorkLimitExceeded { required, limit }
        })?;
        Ok(Self {
            lower,
            width,
            height,
            cells,
            queue_capacity,
        })
    }
}

fn validate_inputs(
    generators: &[VoronoiGenerator],
    radius: f32,
    margin: f32,
    limits: VoronoiLimits,
) -> Result<(), VoronoiError> {
    if !radius.is_finite() {
        return Err(VoronoiError::NonFiniteRadius);
    }
    if radius <= 0.0 {
        return Err(VoronoiError::NonPositiveRadius);
    }
    if !margin.is_finite() {
        return Err(VoronoiError::NonFiniteMargin);
    }
    if margin < 0.0 {
        return Err(VoronoiError::NegativeMargin);
    }
    check_limit(generators.len(), limits.generators, |required, limit| {
        VoronoiError::GeneratorLimitExceeded { required, limit }
    })?;
    for (ordinal, generator) in generators.iter().enumerate() {
        if !generator.center.is_valid() {
            return Err(VoronoiError::NonFiniteGenerator { ordinal });
        }
    }
    Ok(())
}

fn necessary_bounds(generators: &[VoronoiGenerator]) -> Option<(Vec2, Vec2)> {
    let mut bounds: Option<(Vec2, Vec2)> = None;
    for generator in generators.iter().filter(|generator| generator.necessary) {
        bounds = Some(match bounds {
            None => (generator.center, generator.center),
            Some((lower, upper)) => (
                Vec2::new(
                    lower.x.min(generator.center.x),
                    lower.y.min(generator.center.y),
                ),
                Vec2::new(
                    upper.x.max(generator.center.x),
                    upper.y.max(generator.center.y),
                ),
            ),
        });
    }
    bounds
}

fn axis_count(lower: f32, upper: f32, inverse_radius: f32) -> Result<usize, VoronoiError> {
    let scaled_span = inverse_radius * (upper - lower);
    if !scaled_span.is_finite() || scaled_span < 0.0 {
        return Err(VoronoiError::NonFiniteDerivedGeometry);
    }
    if scaled_span >= 2_147_483_648.0 {
        return Err(VoronoiError::AxisCountOutOfRange);
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the finite non-negative f32 is checked below i32::MAX before truncation"
    )]
    let truncated = scaled_span as usize;
    truncated
        .checked_add(1)
        .ok_or(VoronoiError::ArithmeticOverflow)
}

fn work_bound(
    cells: usize,
    generators: usize,
    width: usize,
    height: usize,
) -> Result<usize, VoronoiError> {
    let flood = cells
        .checked_mul(4)
        .and_then(|value| value.checked_add(generators))
        .ok_or(VoronoiError::ArithmeticOverflow)?;
    let boundaries = height
        .checked_mul(width.saturating_sub(1))
        .and_then(|horizontal| {
            width
                .checked_mul(height.saturating_sub(1))
                .and_then(|vertical| horizontal.checked_add(vertical))
        })
        .and_then(|value| value.checked_mul(2))
        .ok_or(VoronoiError::ArithmeticOverflow)?;
    let relaxation = cells
        .checked_mul(generators)
        .and_then(|value| value.checked_mul(4))
        .ok_or(VoronoiError::ArithmeticOverflow)?;
    flood
        .checked_add(boundaries)
        .and_then(|value| value.checked_add(relaxation))
        .ok_or(VoronoiError::ArithmeticOverflow)
}

fn scale_generators(
    generators: &[VoronoiGenerator],
    lower: Vec2,
    inverse_radius: f32,
) -> Result<Vec<ScaledGenerator>, VoronoiError> {
    let mut scaled = Vec::new();
    scaled
        .try_reserve_exact(generators.len())
        .map_err(|_| VoronoiError::AllocationFailed)?;
    for generator in generators {
        let center = inverse_radius * (generator.center - lower);
        if !center.is_valid() {
            return Err(VoronoiError::NonFiniteDerivedGeometry);
        }
        scaled.push(ScaledGenerator {
            center,
            necessary: generator.necessary,
        });
    }
    Ok(scaled)
}

fn allocated_owners(cells: usize) -> Result<Vec<Option<usize>>, VoronoiError> {
    let mut owners = Vec::new();
    owners
        .try_reserve_exact(cells)
        .map_err(|_| VoronoiError::AllocationFailed)?;
    owners.resize(cells, None);
    Ok(owners)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct GridCell {
    x: usize,
    y: usize,
    index: usize,
}

impl GridCell {
    const fn new(x: usize, y: usize, width: usize) -> Self {
        Self {
            x,
            y,
            index: x + y * width,
        }
    }
}

fn neighbor_cells(cell: GridCell, width: usize, height: usize) -> [Option<GridCell>; 4] {
    [
        (cell.x > 0).then(|| GridCell::new(cell.x - 1, cell.y, width)),
        (cell.y > 0).then(|| GridCell::new(cell.x, cell.y - 1, width)),
        (cell.x + 1 < width).then(|| GridCell::new(cell.x + 1, cell.y, width)),
        (cell.y + 1 < height).then(|| GridCell::new(cell.x, cell.y + 1, width)),
    ]
}

#[derive(Debug, Clone, Copy)]
struct Task {
    cell: GridCell,
    generator: usize,
}

struct BoundedQueue {
    tasks: VecDeque<Task>,
    capacity: usize,
}

impl BoundedQueue {
    fn new(capacity: usize) -> Result<Self, VoronoiError> {
        let mut tasks = VecDeque::new();
        tasks
            .try_reserve_exact(capacity)
            .map_err(|_| VoronoiError::AllocationFailed)?;
        Ok(Self { tasks, capacity })
    }

    fn push(&mut self, task: Task) -> Result<(), VoronoiError> {
        if self.tasks.len() >= self.capacity {
            return Err(VoronoiError::QueueLimitExceeded {
                required: self.tasks.len() + 1,
                limit: self.capacity,
            });
        }
        self.tasks.push_back(task);
        Ok(())
    }

    fn pop(&mut self) -> Option<Task> {
        self.tasks.pop_front()
    }
}

struct WorkCounter {
    completed: usize,
    limit: usize,
}

impl WorkCounter {
    const fn new(limit: usize) -> Self {
        Self {
            completed: 0,
            limit,
        }
    }

    fn record(&mut self) -> Result<(), VoronoiError> {
        self.completed = self
            .completed
            .checked_add(1)
            .ok_or(VoronoiError::ArithmeticOverflow)?;
        if self.completed > self.limit {
            return Err(VoronoiError::WorkLimitExceeded {
                required: self.completed,
                limit: self.limit,
            });
        }
        Ok(())
    }
}

fn seed_generators(
    generators: &[ScaledGenerator],
    grid: GridSpec,
    queue: &mut BoundedQueue,
) -> Result<(), VoronoiError> {
    for (ordinal, generator) in generators.iter().enumerate() {
        let Some(x) = grid_coordinate(generator.center.x, grid.width) else {
            continue;
        };
        let Some(y) = grid_coordinate(generator.center.y, grid.height) else {
            continue;
        };
        queue.push(Task {
            cell: GridCell::new(x, y, grid.width),
            generator: ordinal,
        })?;
    }
    Ok(())
}

fn grid_coordinate(value: f32, limit: usize) -> Option<usize> {
    if !value.is_finite() || value < 0.0 {
        return None;
    }
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        reason = "the finite non-negative coordinate is checked inside the grid before truncation"
    )]
    let coordinate = value as usize;
    (coordinate < limit).then_some(coordinate)
}

fn flood_fill(
    owners: &mut [Option<usize>],
    grid: GridSpec,
    queue: &mut BoundedQueue,
    work: &mut WorkCounter,
) -> Result<(), VoronoiError> {
    while let Some(task) = queue.pop() {
        work.record()?;
        if owners[task.cell.index].is_some() {
            continue;
        }
        owners[task.cell.index] = Some(task.generator);
        push_neighbors(task.cell, task.generator, grid, queue)?;
    }
    if owners.iter().any(Option::is_none) {
        return Err(VoronoiError::IncompleteDiagram);
    }
    Ok(())
}

fn push_neighbors(
    cell: GridCell,
    generator: usize,
    grid: GridSpec,
    queue: &mut BoundedQueue,
) -> Result<(), VoronoiError> {
    for neighbor in neighbor_cells(cell, grid.width, grid.height)
        .into_iter()
        .flatten()
    {
        queue.push(Task {
            cell: neighbor,
            generator,
        })?;
    }
    Ok(())
}

fn seed_relaxation(
    owners: &[Option<usize>],
    grid: GridSpec,
    queue: &mut BoundedQueue,
) -> Result<(), VoronoiError> {
    for y in 0..grid.height {
        for x in 0..grid.width.saturating_sub(1) {
            seed_boundary_pair(
                GridCell::new(x, y, grid.width),
                GridCell::new(x + 1, y, grid.width),
                owners,
                queue,
            )?;
        }
    }
    for y in 0..grid.height.saturating_sub(1) {
        for x in 0..grid.width {
            seed_boundary_pair(
                GridCell::new(x, y, grid.width),
                GridCell::new(x, y + 1, grid.width),
                owners,
                queue,
            )?;
        }
    }
    Ok(())
}

fn seed_boundary_pair(
    first: GridCell,
    second: GridCell,
    owners: &[Option<usize>],
    queue: &mut BoundedQueue,
) -> Result<(), VoronoiError> {
    let first_owner = owners[first.index].ok_or(VoronoiError::IncompleteDiagram)?;
    let second_owner = owners[second.index].ok_or(VoronoiError::IncompleteDiagram)?;
    if first_owner == second_owner {
        return Ok(());
    }
    queue.push(Task {
        cell: first,
        generator: second_owner,
    })?;
    queue.push(Task {
        cell: second,
        generator: first_owner,
    })
}

fn relax_distances(
    owners: &mut [Option<usize>],
    generators: &[ScaledGenerator],
    grid: GridSpec,
    queue: &mut BoundedQueue,
    work: &mut WorkCounter,
) -> Result<(), VoronoiError> {
    while let Some(task) = queue.pop() {
        work.record()?;
        let incumbent = owners[task.cell.index].ok_or(VoronoiError::IncompleteDiagram)?;
        if incumbent == task.generator {
            continue;
        }
        let incumbent_distance = squared_cell_distance(generators[incumbent], task.cell);
        let candidate_distance = squared_cell_distance(generators[task.generator], task.cell);
        if incumbent_distance > candidate_distance {
            owners[task.cell.index] = Some(task.generator);
            push_neighbors(task.cell, task.generator, grid, queue)?;
        }
    }
    Ok(())
}

fn squared_cell_distance(generator: ScaledGenerator, cell: GridCell) -> f32 {
    #[allow(
        clippy::cast_precision_loss,
        reason = "grid axes are checked below i32::MAX, matching the source int32-to-f32 conversion"
    )]
    let (x, y) = (cell.x as f32, cell.y as f32);
    let dx = generator.center.x - x;
    let dy = generator.center.y - y;
    dx * dx + dy * dy
}

fn collect_nodes(
    owners: &[Option<usize>],
    generators: &[ScaledGenerator],
    grid: GridSpec,
    maximum_nodes: usize,
) -> Result<Vec<VoronoiNode>, VoronoiError> {
    let mut nodes = Vec::new();
    nodes
        .try_reserve_exact(
            grid.width
                .saturating_sub(1)
                .checked_mul(grid.height.saturating_sub(1))
                .and_then(|count| count.checked_mul(2))
                .ok_or(VoronoiError::ArithmeticOverflow)?,
        )
        .map_err(|_| VoronoiError::AllocationFailed)?;
    for y in 0..grid.height.saturating_sub(1) {
        for x in 0..grid.width.saturating_sub(1) {
            let index = x + y * grid.width;
            let [a, b, c, d] = [
                owner(owners, index)?,
                owner(owners, index + 1)?,
                owner(owners, index + grid.width)?,
                owner(owners, index + 1 + grid.width)?,
            ];
            if b == c {
                continue;
            }
            maybe_push_node(&mut nodes, [a, b, c], generators, maximum_nodes)?;
            maybe_push_node(&mut nodes, [b, d, c], generators, maximum_nodes)?;
        }
    }
    Ok(nodes)
}

fn owner(owners: &[Option<usize>], index: usize) -> Result<usize, VoronoiError> {
    owners[index].ok_or(VoronoiError::IncompleteDiagram)
}

fn maybe_push_node(
    nodes: &mut Vec<VoronoiNode>,
    ordinals: [usize; 3],
    generators: &[ScaledGenerator],
    maximum_nodes: usize,
) -> Result<(), VoronoiError> {
    let [a, b, c] = ordinals;
    if a == b || a == c || b == c {
        return Ok(());
    }
    if !(generators[a].necessary || generators[b].necessary || generators[c].necessary) {
        return Ok(());
    }
    if nodes.len() >= maximum_nodes {
        return Err(VoronoiError::NodeLimitExceeded {
            required: nodes.len() + 1,
            limit: maximum_nodes,
        });
    }
    nodes.push(VoronoiNode {
        generator_ordinals: ordinals,
    });
    Ok(())
}

fn check_limit(
    required: usize,
    limit: usize,
    error: impl FnOnce(usize, usize) -> VoronoiError,
) -> Result<(), VoronoiError> {
    if required > limit {
        return Err(error(required, limit));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
