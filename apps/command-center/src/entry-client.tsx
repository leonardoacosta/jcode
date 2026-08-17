import { mount, StartClient } from "@solidjs/start/client";

document.documentElement.dataset.commandCenterBuild = "route-reactivity-v1";
mount(() => <StartClient />, document.getElementById("app")!);
