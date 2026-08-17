import { mount, StartClient } from "@solidjs/start/client";

document.documentElement.dataset.commandCenterBuild = "route-reactivity-v1";
export default mount(() => <StartClient />, document.getElementById("app")!);
