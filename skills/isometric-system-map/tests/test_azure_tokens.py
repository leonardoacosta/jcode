import json
import re
import unittest
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
AZURE_TOKENS = ROOT / "assets" / "azure-tokens.json"
AZURE_SPRITE = ROOT / "assets" / "azure-icons.svg"


class AzureTokenContractTests(unittest.TestCase):
    def test_family_icon_fallbacks_cover_declared_and_resource_type_families(self):
        tokens = json.loads(AZURE_TOKENS.read_text())
        sprite_symbols = set(re.findall(r'<symbol\s+id="([^"]+)"', AZURE_SPRITE.read_text()))

        families = set(tokens.get("families", {}))
        resource_type_families = set(tokens.get("resource_type_family", {}).values())
        fallbacks = tokens.get("family_icon_fallbacks")

        self.assertIsInstance(
            fallbacks,
            dict,
            "assets/azure-tokens.json must define package-owned family_icon_fallbacks",
        )
        self.assertEqual(
            families,
            set(fallbacks),
            "family_icon_fallbacks must provide exactly one fallback for every declared family",
        )
        self.assertEqual(
            set(),
            resource_type_families - families,
            "every resource_type_family value must name a declared family",
        )
        self.assertEqual(
            set(),
            resource_type_families - set(fallbacks),
            "every resource_type_family value must name a family with a fallback",
        )
        missing_symbols = {
            family: symbol_id
            for family, symbol_id in fallbacks.items()
            if symbol_id not in sprite_symbols
        }
        self.assertEqual(
            {},
            missing_symbols,
            "each family fallback must be an admitted symbol id from assets/azure-icons.svg",
        )


if __name__ == "__main__":
    unittest.main()
