# Graph Report - ast-grep  (2026-11-25)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 1957 nodes · 5272 edges · 89 communities (77 shown, 12 thin omitted)
- Extraction: 99% EXTRACTED · 1% INFERRED · 0% AMBIGUOUS · INFERRED: 69 edges (avg confidence: 0.73)
- Token cost: 0 input · 0 output

## Community Hubs (Navigation)
- Community 0
- Community 1
- Community 2
- Community 3
- Community 4
- Community 5
- Community 6
- Community 7
- Community 8
- Community 9
- Community 10
- Community 11
- Community 12
- Community 13
- Community 14
- Community 15
- Community 16
- Community 17
- Community 18
- Community 19
- Community 20
- Community 21
- Community 22
- Community 23
- Community 24
- Community 25
- Community 26
- Community 27
- Community 28
- Community 29
- Community 30
- Community 31
- Community 32
- Community 33
- Community 34
- Community 35
- Community 36
- Community 37
- Community 38
- Community 39
- Community 40
- Community 41
- Community 42
- Community 43
- Community 44
- Community 45
- Community 46
- Community 47
- Community 48
- Community 49
- Community 50
- Community 51
- Community 52
- Community 53
- Community 54
- Community 55
- Community 56
- Community 57
- Community 58
- Community 59
- Community 60
- Community 61
- Community 62
- Community 63
- Community 64
- Community 65
- Community 66
- Community 67
- Community 68
- Community 69
- Community 70
- Community 71
- Community 72
- Community 73
- Community 74
- Community 75
- Community 76
- Community 77
- Community 78
- Community 79
- Community 80
- Community 81
- Community 82
- Community 83
- Community 84
- Community 85
- Community 86
- Community 88

## God Nodes (most connected - your core abstractions)
1. `Node` - 162 edges
2. `Rule` - 99 edges
3. `MetaVarEnv` - 72 edges
4. `RuleSerializeError` - 55 edges
5. `Matcher` - 54 edges
6. `Pattern` - 50 edges
7. `SelectorError` - 48 edges
8. `DeserializeEnv` - 47 edges
9. `RuleCore` - 47 edges
10. `Doc` - 46 edges

## Surprising Connections (you probably didn't know these)
- `TypeScript` --implements--> `LanguageExt`  [EXTRACTED]
  config/src/lib.rs → core/src/tree_sitter/mod.rs
- `RuleCore` --implements--> `Matcher`  [EXTRACTED]
  config/src/rule_core.rs → core/src/matcher.rs
- `Rule` --implements--> `Matcher`  [EXTRACTED]
  config/src/rule/mod.rs → core/src/matcher.rs
- `NthChild` --implements--> `Matcher`  [EXTRACTED]
  config/src/rule/nth_child.rs → core/src/matcher.rs
- `ReferentRule` --implements--> `Matcher`  [EXTRACTED]
  config/src/rule/referent_rule.rs → core/src/matcher.rs

## Import Cycles
- 1-file cycle: `config/src/transform/string_case.rs -> config/src/transform/string_case.rs`
- 2-file cycle: `core/src/matcher.rs -> core/src/node.rs -> core/src/matcher.rs`
- 2-file cycle: `core/src/matcher.rs -> core/src/matcher/pattern.rs -> core/src/matcher.rs`
- 2-file cycle: `core/src/matcher.rs -> core/src/matcher/kind.rs -> core/src/matcher.rs`
- 2-file cycle: `core/src/matcher.rs -> core/src/meta_var.rs -> core/src/matcher.rs`
- 2-file cycle: `core/src/matcher.rs -> core/src/matcher/node_match.rs -> core/src/matcher.rs`
- 2-file cycle: `core/src/matcher.rs -> core/src/matcher/text.rs -> core/src/matcher.rs`
- 2-file cycle: `config/src/combined.rs -> config/src/lib.rs -> config/src/combined.rs`
- 2-file cycle: `core/src/tree_sitter/mod.rs -> core/src/tree_sitter/traversal.rs -> core/src/tree_sitter/mod.rs`
- 2-file cycle: `config/src/fixer.rs -> config/src/lib.rs -> config/src/fixer.rs`
- 2-file cycle: `config/src/label.rs -> config/src/lib.rs -> config/src/label.rs`
- 2-file cycle: `config/src/lib.rs -> config/src/rule_collection.rs -> config/src/lib.rs`
- 2-file cycle: `config/src/lib.rs -> config/src/rule/mod.rs -> config/src/lib.rs`
- 2-file cycle: `config/src/lib.rs -> config/src/rule/selector.rs -> config/src/lib.rs`
- 2-file cycle: `config/src/lib.rs -> config/src/rule/deserialize_env.rs -> config/src/lib.rs`
- 2-file cycle: `config/src/lib.rs -> config/src/rule_core.rs -> config/src/lib.rs`
- 2-file cycle: `config/src/lib.rs -> config/src/rule_config.rs -> config/src/lib.rs`
- 2-file cycle: `config/src/lib.rs -> config/src/rule/referent_rule.rs -> config/src/lib.rs`
- 2-file cycle: `config/src/lib.rs -> config/src/transform/mod.rs -> config/src/lib.rs`
- 2-file cycle: `config/src/rule/deserialize_env.rs -> config/src/rule/mod.rs -> config/src/rule/deserialize_env.rs`

## Communities (89 total, 12 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.06
Nodes (35): MatchAll, All, All<P>, And, Any, Any<M>, Any<P>, find_all() (+27 more)

### Community 1 - "Community 1"
Cohesion: 0.06
Nodes (21): AstGrep<StrDoc<L>>, ContentExt, crate::Node<'r, StrDoc<L>>, DisplayContext, LanguageExt, parse(), parse_lang(), perform_edit() (+13 more)

### Community 2 - "Community 2"
Cohesion: 0.10
Nodes (52): Input, Input<'a, L>, input_to_tokens(), parse_complex_selector(), parse_compound_selector(), parse_has_argument(), parse_not_argument(), parse_nth_child_argument() (+44 more)

### Community 3 - "Community 3"
Cohesion: 0.06
Nodes (4): SgNode, Node<'r>, NodeWalker, NodeWalker<'tree>

### Community 4 - "Community 4"
Cohesion: 0.07
Nodes (32): deser(), find_index(), FunctionalPosition, NthChild, NthChildError, IllegalCharacter, InvalidRule, InvalidSyntax (+24 more)

### Community 5 - "Community 5"
Cohesion: 0.12
Nodes (29): DependentRule, DeserializeEnv<L>, get_dependent_utils(), into_map(), OrderResult, ParsedGlobalRule, ParsedGlobalRule<L>, Result (+21 more)

### Community 6 - "Community 6"
Cohesion: 0.06
Nodes (16): inclusive_until(), cast_err!, parse_stop_by(), SerializableStopBy, End, .END_KEY, Neighbor, .NEIGHBOR_KEY (+8 more)

### Community 7 - "Community 7"
Cohesion: 0.11
Nodes (24): expand_end(), expand_start(), Expansion, Fixer, FixerError, InvalidRewriter, InvalidTemplate, MissingTitle (+16 more)

### Community 8 - "Community 8"
Cohesion: 0.10
Nodes (28): create_template(), gen_replacement(), Indent, maybe_get_var(), MultiByteMetaVar, replace_fixer(), Template, TemplateFix (+20 more)

### Community 9 - "Community 9"
Cohesion: 0.12
Nodes (24): Root, StrDoc, Algorithm, CASES, Level, MATCHER_CASES, Post, post_order() (+16 more)

### Community 10 - "Community 10"
Cohesion: 0.08
Nodes (10): Correct, ERROR_STR, Maybe, Absent, Present, Maybe<T>, Option<T>, test_from_optio() (+2 more)

### Community 11 - "Community 11"
Cohesion: 0.17
Nodes (27): deserialize_relational_rule(), find_rule(), Follows, Has, Inside, make_rule(), Precedes, test_follows_immediate() (+19 more)

### Community 12 - "Community 12"
Cohesion: 0.12
Nodes (28): collect_vars(), defined_vars(), get_kind(), match_env(), meta_var_name(), test_bare_wildcard(), test_class_assignment(), test_contextual_potential_kinds() (+20 more)

### Community 13 - "Community 13"
Cohesion: 0.11
Nodes (21): from_yaml_string(), TypeScript, Tsx, assert_ignore_path(), assert_match_path(), build_glob_set(), ContingentRule, make_rule() (+13 more)

### Community 14 - "Community 14"
Cohesion: 0.09
Nodes (22): get_matches_config(), make_undefined_error(), RULE_CONFIG_FIELDS, test_add_rewriters(), test_complex_metadata(), test_get_fixer(), test_get_message_transform(), test_get_message_transform_string() (+14 more)

### Community 15 - "Community 15"
Cohesion: 0.12
Nodes (27): create_rule(), get_suppression_kind(), IGNORE_TEXT, is_suppression_comment(), parse_suppression_set(), SuppressKind, File, Line (+19 more)

### Community 16 - "Community 16"
Cohesion: 0.13
Nodes (27): CONVERT_CASE, decompose_args(), decompose_str(), DecomposedTransString, ParseTransError, ArgValue, InvalidArg, InvalidTransform (+19 more)

### Community 17 - "Community 17"
Cohesion: 0.12
Nodes (11): Tsx, Node<'static, D>, NodeData, NodeMatch<'static, D>, PinnedNodeData, PinnedNodeData<D, T>, return_from_func(), return_vec() (+3 more)

### Community 18 - "Community 18"
Cohesion: 0.14
Nodes (11): ComputeEnd, Cow<'_, MetaVarEnv<'t, D>>, find_end_recursive(), match_end_non_recursive(), match_leaf_meta_var(), match_node_non_recursive(), test_root_metavar_matches_comment(), MatcherExt (+3 more)

### Community 19 - "Community 19"
Cohesion: 0.10
Nodes (17): AtomicRule, Categorized, CompositeRule, deserialize_rule(), PatternStyle, Contextual, Str, RelationalRule (+9 more)

### Community 20 - "Community 20"
Cohesion: 0.11
Nodes (9): borrow_node(), Edit, Node<'tree, D>, NodeMatch<'_, D>, NodeMatch<'tree, D>, test_node_env(), test_node_match_as_node(), test_replace_by() (+1 more)

### Community 21 - "Community 21"
Cohesion: 0.09
Nodes (20): deserialize_matches_rule(), deserialze_atomic_rule(), deserialze_composite_rule(), Rule, All, Any, Follows, Has (+12 more)

### Community 22 - "Community 22"
Cohesion: 0.14
Nodes (12): convert_node_to_pattern(), extract_var_from_node(), is_single_node(), Pattern, PatternBuilder, PatternError, InvalidKind, MultipleNode (+4 more)

### Community 24 - "Community 24"
Cohesion: 0.15
Nodes (23): deindent_slice(), DeindentedExtract, MultiLine, SingleLine, extract_with_deindent(), formatted_slice(), indent_lines(), indent_lines_impl() (+15 more)

### Community 25 - "Community 25"
Cohesion: 0.17
Nodes (20): NO_SUPPRESS_ALL_ID, from_multiple(), from_str(), make_yaml(), test_deserialize_constraints(), test_deserialize_inside(), test_deserialize_kind(), test_deserialize_meta_var() (+12 more)

### Community 26 - "Community 26"
Cohesion: 0.13
Nodes (14): DeserializeEnv, RuleSerializeError, FieldNotSupported, InvalidField, InvalidKind, InvalidPattern, InvalidRange, InvalidUtils (+6 more)

### Community 27 - "Community 27"
Cohesion: 0.17
Nodes (3): Edit, Node<'_, D>, Root<D>

### Community 28 - "Community 28"
Cohesion: 0.15
Nodes (23): get_matcher(), get_matcher_with_globals(), get_rewriters(), test_augmented_rule(), test_bare_parameterized_global_rule_without_args_is_rejected(), test_constraints_writing_to_env(), test_cyclic_transform_error(), test_local_util_metavar_does_affect_yaml_rule_matching() (+15 more)

### Community 29 - "Community 29"
Cohesion: 0.16
Nodes (6): RResult, RuleCore, SerializableRuleCore, test_constraints_inheriting_env(), test_rule_reg_with_utils(), test_rule_with_constraints()

### Community 30 - "Community 30"
Cohesion: 0.15
Nodes (12): are_kinds_matching(), is_error_kind(), KindMatcher, KindMatcherError, InvalidKindName, pattern_node(), test_kind_match(), test_kind_non_match() (+4 more)

### Community 31 - "Community 31"
Cohesion: 0.16
Nodes (16): check_fix(), check_rewriter_fix(), check_rule_with_hint(), check_utils_defined(), check_var_in_constraints(), check_var_in_fix(), check_var_in_transform(), check_vars() (+8 more)

### Community 32 - "Community 32"
Cohesion: 0.14
Nodes (17): RuleConfigError, Core, Fixer, LabelVariable, MissingPotentialKinds, Rewriter, UndefinedRewriter, Yaml (+9 more)

### Community 33 - "Community 33"
Cohesion: 0.13
Nodes (7): GlobalTemplate, parameterized_potential_kinds(), verify_parameterized_referent(), match_global_rule(), UndefinedUtil, Registration<R>, RegistrationRef

### Community 34 - "Community 34"
Cohesion: 0.15
Nodes (14): RangeMatcher, RangeMatcherError, InvalidRange, SerializablePosition, SerializableRange, test_invalid_range(), test_multiline_range(), test_range_match() (+6 more)

### Community 35 - "Community 35"
Cohesion: 0.21
Nodes (22): find_node_recursive(), test_ellipsis_end(), test_end(), test_gh_1087(), test_leading_ellipsis(), test_leading_ellipsis_metavar_anchor(), test_match(), test_match_end() (+14 more)

### Community 36 - "Community 36"
Cohesion: 0.18
Nodes (10): CombinedScan, MaySuppressed, No, Yes, ScanResult, ScanResultInner, ScanResultInner<'t, D>, Suppression (+2 more)

### Community 37 - "Community 37"
Cohesion: 0.18
Nodes (6): match_and_add_label(), test_not_in_relation_does_not_leak_env(), MetaVarEnv, And<P1, P2>, Not<P>, Or<P1, P2>

### Community 38 - "Community 38"
Cohesion: 0.20
Nodes (14): BindingFrame, export_vars(), lookup_bound_rule(), match_bound_rule(), match_global_template(), match_parameterized_referent(), SerializableUtilityArgs, SerializableUtilityCall (+6 more)

### Community 39 - "Community 39"
Cohesion: 0.27
Nodes (16): RuleRegistration, apply_transformation(), compute_rewritten(), str_vec!, make_rewriter_reg(), make_rewriters(), test_ignore_unused_rewriters(), test_node_not_found() (+8 more)

### Community 40 - "Community 40"
Cohesion: 0.24
Nodes (17): follows(), MULTI_LINE, precedes(), test_ascii_pos(), test_child_by_field_id(), test_display_context(), test_empty(), test_field() (+9 more)

### Community 41 - "Community 41"
Cohesion: 0.14
Nodes (5): Level<'tree, L>, Post<'tree, L>, Pre<'tree, L>, TsPre<'tree>, Visit<'t, D, T, M>

### Community 42 - "Community 42"
Cohesion: 0.13
Nodes (13): RewriterError, RewriterErrorReason, Core, Fixer, NoFixInRewriter, SerializableRewriter, RuleCoreError, Constraints (+5 more)

### Community 43 - "Community 43"
Cohesion: 0.25
Nodes (5): ContingentRule<L>, RuleBucket, RuleBucket<L>, RuleCollection<L>, RuleConfig

### Community 44 - "Community 44"
Cohesion: 0.15
Nodes (14): TransformError, AlreadyDefined, Cyclic, MalformedVar, Parse, Rewrite, Rewrite<String>, parse_meta_var() (+6 more)

### Community 45 - "Community 45"
Cohesion: 0.15
Nodes (16): assert_format(), assert_split(), assert_split_sep(), CAMEL, DOT, KEBAB, PASCAL, PATH (+8 more)

### Community 46 - "Community 46"
Cohesion: 0.13
Nodes (14): CaseState, IgnoreCase, Lower, MultiUpper, OneUpper, Delimiter, Separator, CaseChange (+6 more)

### Community 47 - "Community 47"
Cohesion: 0.18
Nodes (13): extract_meta_var(), extract_var(), match_constraints(), MetaVariable, Capture, Dropped, MultiCapture, Multiple (+5 more)

### Community 48 - "Community 48"
Cohesion: 0.16
Nodes (9): post_order_with_matcher(), pre_order_with_matcher(), test_post_order_visitor(), test_pre_order_visitor(), test_traversal_leaf(), Visit, Visitor, Visitor<M> (+1 more)

### Community 49 - "Community 49"
Cohesion: 0.28
Nodes (9): ReferentRuleError, CyclicRule, DuplicateRule, Result, test_cyclic_all(), test_cyclic_error(), test_cyclic_not(), test_recursive_relation_potential_kinds_terminates() (+1 more)

### Community 50 - "Community 50"
Cohesion: 0.21
Nodes (12): get_default_labels(), get_labels_from_config(), Label, Label<'_, 't, D>, LabelConfig, LabelStyle, Primary, Secondary (+4 more)

### Community 51 - "Community 51"
Cohesion: 0.34
Nodes (17): get_transformed(), parse(), R, test_capitalize_convert(), test_dependent_trans(), test_insert_env(), test_lowercase_convert(), test_no_end_char() (+9 more)

### Community 52 - "Community 52"
Cohesion: 0.22
Nodes (13): collect_edits(), gen_replacement(), get_meta_var_replacement(), merge_edits_to_vec(), test_ellipsis_meta_var(), test_ellipsis_replace(), test_multi_ellipsis(), test_multiple_env() (+5 more)

### Community 53 - "Community 53"
Cohesion: 0.26
Nodes (4): does_node_match_exactly(), get_var_bytes_impl(), HashMap<String, String>, MetaVarEnv<'t, D>

### Community 54 - "Community 54"
Cohesion: 0.20
Nodes (9): check_one_rewriter_in_rule(), check_rewriters_in_transform(), Rewriter, Bytes, find_and_make_edits(), get_nodes_from_env(), make_edit(), replace_one() (+1 more)

### Community 55 - "Community 55"
Cohesion: 0.29
Nodes (12): match_ellipsis(), match_node_impl(), match_nodes_impl_recursive(), match_root_node_impl(), match_single_node_while_skip_trivial(), may_match_ellipsis_impl(), try_get_ellipsis_mode(), Aggregator (+4 more)

### Community 56 - "Community 56"
Cohesion: 0.17
Nodes (9): MatchStrictness, Ast, Cst, Relaxed, Signature, Smart, Template, skip_comment() (+1 more)

### Community 57 - "Community 57"
Cohesion: 0.17
Nodes (8): CombinedScan<'r, L>, SerializableRuleConfig, Severity, Error, Hint, Info, Off, Warning

### Community 58 - "Community 58"
Cohesion: 0.19
Nodes (5): test_cyclic_transform(), test_single_cyclic_transform(), test_transform_indentation(), test_transform_use_matched(), Transform

### Community 59 - "Community 59"
Cohesion: 0.18
Nodes (7): Underlying, get_indent_at_offset(), get_new_line(), get_space(), remove_indent(), Content, String

### Community 60 - "Community 60"
Cohesion: 0.25
Nodes (7): Ctx, Convert<MetaVariable>, get_text_from_env(), Replace<MetaVariable>, resolve_char(), Substring<MetaVariable>, Trans<MetaVariable>

### Community 61 - "Community 61"
Cohesion: 0.19
Nodes (12): lower_utility_args(), lower_utility_call(), ParameterizedUtilError, DuplicateUtilityArgument, InvalidUtilityArgument, InvalidUtilityCall, MissingUtilityArgument, MissingUtilityArguments (+4 more)

### Community 63 - "Community 63"
Cohesion: 0.35
Nodes (6): NodeMatch, Node<'_, D>, Replacer, Root<D>, str, &T

### Community 64 - "Community 64"
Cohesion: 0.19
Nodes (3): RegexMatcher, RegexMatcherError, Regex

### Community 65 - "Community 65"
Cohesion: 0.26
Nodes (9): is_valid_first_char(), is_valid_meta_var_char(), MetaVariableID, Edit, MetaVarExtract, Multiple, Single, Transformed (+1 more)

### Community 67 - "Community 67"
Cohesion: 0.23
Nodes (6): ReferentArgs, ReferentFormat, Args, IdRef, Param, ReferentRule

### Community 68 - "Community 68"
Cohesion: 0.36
Nodes (9): match_tree(), matched(), test_ast_match(), test_cst_match(), test_relaxed_match(), test_signature_match(), test_smart_match(), test_template_match() (+1 more)

### Community 69 - "Community 69"
Cohesion: 0.27
Nodes (6): relaxed_pattern(), template_pattern(), test_ast_trailing_comma(), test_ignore_comment(), test_match(), test_template_pattern()

### Community 70 - "Community 70"
Cohesion: 0.24
Nodes (8): pattern_node(), test_bare_wildcard_in_context(), test_contextual_match_with_env(), test_contextual_pattern(), test_contextual_pattern_vars(), test_contextual_unmatch_with_env(), test_pattern_should_not_pollute_env(), test_template_pattern_have_no_kinds()

### Community 73 - "Community 73"
Cohesion: 0.22
Nodes (8): MatchStrictness, Strictness, Ast, Cst, Relaxed, Signature, Smart, Template

### Community 74 - "Community 74"
Cohesion: 0.27
Nodes (3): MatchNone, dump_pattern_impl(), DumpPattern

### Community 75 - "Community 75"
Cohesion: 0.42
Nodes (4): Prune<'tree, L>, PruneSubtree, test_prune_pre_order_skips_subtree(), test_prune_subtree_scope_tracks_exit()

### Community 77 - "Community 77"
Cohesion: 0.25
Nodes (8): StringCase, CamelCase, Capitalize, KebabCase, LowerCase, PascalCase, SnakeCase, UpperCase

### Community 78 - "Community 78"
Cohesion: 0.25
Nodes (3): Convert, Substring, Trans<T>

### Community 79 - "Community 79"
Cohesion: 0.43
Nodes (6): Result, test_replace(), test_replace_by_rule(), test_replace_trivia(), test_replace_trivia_with_skipped(), test_replace_unnamed_node()

### Community 80 - "Community 80"
Cohesion: 0.33
Nodes (6): MatchOneNode, MatchedBoth, NoMatch, SkipBoth, SkipCandidate, SkipGoal

### Community 81 - "Community 81"
Cohesion: 0.60
Nodes (3): capitalize(), join(), join_camel_case()

### Community 84 - "Community 84"
Cohesion: 0.50
Nodes (4): ControlFlow, Continue, Fallthrough, Return

## Knowledge Gaps
- **157 isolated node(s):** `Def<M>`, `Trans<T>`, `Visitor<M>`, `Op<And<M, N>>`, `Op<Or<M, N>>` (+152 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 388 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **12 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Node` connect `Community 18` to `Community 0`, `Community 4`, `Community 6`, `Community 9`, `Community 11`, `Community 12`, `Community 15`, `Community 17`, `Community 19`, `Community 20`, `Community 22`, `Community 23`, `Community 27`, `Community 28`, `Community 29`, `Community 30`, `Community 33`, `Community 34`, `Community 35`, `Community 36`, `Community 37`, `Community 38`, `Community 39`, `Community 40`, `Community 41`, `Community 47`, `Community 48`, `Community 49`, `Community 50`, `Community 52`, `Community 53`, `Community 54`, `Community 55`, `Community 56`, `Community 63`, `Community 64`, `Community 65`, `Community 68`, `Community 69`, `Community 70`, `Community 71`, `Community 72`, `Community 74`, `Community 75`, `Community 79`, `Community 86`?**
  _High betweenness centrality (0.235) - this node is a cross-community bridge._
- **Are the 2 inferred relationships involving `Node` (e.g. with `.get_node_text()` and `.root_node()`) actually correct?**
  _`Node` has 2 INFERRED edges - model-reasoned connections that need verification._
- **What connects `Def<M>`, `Trans<T>`, `Visitor<M>` to the rest of the system?**
  _157 weakly-connected nodes found - possible missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.05875251509054326 - nodes in this community are weakly interconnected._
- **Why does `Matcher` connect `Community 11` to `Community 0`, `Community 1`, `Community 4`, `Community 5`, `Community 7`, `Community 9`, `Community 12`, `Community 14`, `Community 15`, `Community 18`, `Community 19`, `Community 20`, `Community 21`, `Community 22`, `Community 28`, `Community 29`, `Community 30`, `Community 37`, `Community 38`, `Community 40`, `Community 47`, `Community 49`, `Community 63`, `Community 64`, `Community 65`, `Community 67`, `Community 68`, `Community 74`, `Community 79`?**
  _High betweenness centrality (0.065) - this node is a cross-community bridge._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.06331976481230213 - nodes in this community are weakly interconnected._
- **Why does `Doc` connect `Community 72` to `Community 0`, `Community 1`, `Community 4`, `Community 6`, `Community 7`, `Community 8`, `Community 9`, `Community 11`, `Community 12`, `Community 14`, `Community 15`, `Community 17`, `Community 18`, `Community 19`, `Community 20`, `Community 28`, `Community 30`, `Community 34`, `Community 35`, `Community 38`, `Community 39`, `Community 40`, `Community 47`, `Community 49`, `Community 50`, `Community 51`, `Community 52`, `Community 56`, `Community 58`, `Community 64`, `Community 65`, `Community 68`, `Community 69`, `Community 74`, `Community 79`?**
  _High betweenness centrality (0.062) - this node is a cross-community bridge._