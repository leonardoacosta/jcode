import ast
import subprocess
import tempfile
import unittest
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[3]
SKILL_ROOT = Path(__file__).resolve().parents[1]
RENDERER = SKILL_ROOT / "scripts" / "render_canvas.py"
DIRECTIONAL_SCENE = SKILL_ROOT / "tests" / "fixtures" / "directional-scene.json"
DIRECTIONAL_VIEWS = SKILL_ROOT / "tests" / "fixtures" / "directional-views.json"
AZURE_THEME = SKILL_ROOT / "themes" / "azure-topology.js"
VERIFY_CLI = SKILL_ROOT / "scripts" / "verify_views_browser.py"
CHROMIUM = Path("/usr/bin/chromium")


class VerifyViewsBrowserCliContractTests(unittest.TestCase):
    def render_directional_views_artifact(self, output: Path) -> None:
        result = subprocess.run(
            [
                "python3",
                str(RENDERER),
                str(DIRECTIONAL_SCENE),
                str(AZURE_THEME),
                str(output),
                "--views",
                str(DIRECTIONAL_VIEWS),
            ],
            cwd=REPO_ROOT,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(
            result.returncode,
            0,
            "temporary directional views-enabled Azure artifact must render before browser CLI verification\n"
            f"STDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}",
        )
        self.assertTrue(output.exists(), "renderer did not create the temporary artifact")

    def assert_verify_cli_is_stdlib_only(self) -> None:
        source = VERIFY_CLI.read_text(encoding="utf-8")
        lowered = source.lower()
        for forbidden in ("selenium", "playwright", "puppeteer"):
            self.assertNotIn(forbidden, lowered)

        tree = ast.parse(source, filename=str(VERIFY_CLI))
        imports = set()
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                imports.update(alias.name.split(".", 1)[0] for alias in node.names)
            elif isinstance(node, ast.ImportFrom) and node.module:
                imports.add(node.module.split(".", 1)[0])

        third_party = sorted(
            name
            for name in imports
            if name not in set(__import__("sys").stdlib_module_names) and name != "__future__"
        )
        self.assertEqual(third_party, [], "verify_views_browser.py must import stdlib modules only")

    def test_cli_verifies_directional_views_in_file_and_loopback_browser_contexts(self):
        self.assertTrue(CHROMIUM.exists(), "expected chromium at /usr/bin/chromium for the public CLI contract")

        with tempfile.TemporaryDirectory() as directory:
            artifact = Path(directory) / "directional-views.html"
            self.render_directional_views_artifact(artifact)

            result = subprocess.run(
                [
                    "python3",
                    str(VERIFY_CLI.relative_to(REPO_ROOT)),
                    "--chromium",
                    str(CHROMIUM),
                    "--artifact",
                    str(artifact),
                ],
                cwd=REPO_ROOT,
                capture_output=True,
                text=True,
                check=False,
            )

        self.assertEqual(
            result.returncode,
            0,
            "verify_views_browser.py should exit 0 with structured PASS output\n"
            f"STDOUT:\n{result.stdout}\nSTDERR:\n{result.stderr}",
        )
        output = result.stdout
        required_fragments = [
            "PASS",
            "file://",
            "loopback HTTP",
            "Runtime fragment",
            "Network fragment",
            "ADO fragment",
            "native keyboard tab behavior",
            "no-JS document order",
            "desktop viewport",
            "320px viewport",
            "200% zoom",
            "reduced motion",
            "focusability",
            "selection retention",
            "direct semantics",
            "inferred semantics",
            "held semantics",
            "zero console errors",
            "zero page errors",
            "zero horizontal clipping",
            "zero unexpected network requests",
        ]
        missing = [fragment for fragment in required_fragments if fragment not in output]
        self.assertEqual(missing, [], "structured PASS output is missing expected checks")
        self.assert_verify_cli_is_stdlib_only()


if __name__ == "__main__":
    unittest.main()
