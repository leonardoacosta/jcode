# scripts/test_safety_budgets.py

- load · function · L12-L16 — def load(name)
- ProductionLinesTests · class · L19-L118 — class ProductionLinesTests(unittest.TestCase)
- test_cfg_test_qualified_functions_do_not_leak_into_production · method · L20-L35 — def test_cfg_test_qualified_functions_do_not_leak_into_production(self)
- test_regressions_still_fail_without_writing_baselines · method · L37-L54 — def test_regressions_still_fail_without_writing_baselines(self)
- test_inline_cfg_and_multiline_signature_do_not_hide_next_production_item · method · L56-L65 — def test_inline_cfg_and_multiline_signature_do_not_hide_next_production_item(self)
- test_literals_and_comments_do_not_count_or_unbalance_test_blocks · method · L67-L86 — def test_literals_and_comments_do_not_count_or_unbalance_test_blocks(self)
- test_external_test_modules_paths_nested_and_shared_references · method · L88-L110 — def test_external_test_modules_paths_nested_and_shared_references(self)
- test_non_test_cfg_and_unconditional_async_remain_counted · method · L112-L118 — def test_non_test_cfg_and_unconditional_async_remain_counted(self)
