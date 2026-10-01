// Copies the repository's specification schema into the package, where the server serves it as
// chartlet://schema. Run before tests and packing; the copy is not committed.

import { copyFileSync, mkdirSync } from "node:fs";

const source = new URL("../../../schema/chartlet.schema.json", import.meta.url);
const target = new URL("../schema/chartlet.schema.json", import.meta.url);

mkdirSync(new URL(".", target), { recursive: true });
copyFileSync(source, target);
