import { render } from "solid-js/web";

import { App } from "./App";
import "./styles/tailwind.css";
import "./app.css";
import "./styles/player.css";
import "./styles/responsive.css";
import "./styles/canvas-stage.css";

const root = document.getElementById("root");
if (root === null) {
  throw new Error("Application root is unavailable");
}

render(() => <App />, root);
