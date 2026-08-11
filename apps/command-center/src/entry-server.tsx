import { createHandler, StartServer } from "@solidjs/start/server";
import Document from "./root";

export default createHandler(() => <StartServer document={Document} />);
