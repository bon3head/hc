# Graph Report - ripgrep  (2026-11-25)

## Corpus Check
- cluster-only mode — file stats not available

## Summary
- 1022 nodes · 2851 edges · 34 communities (26 shown, 8 thin omitted)
- Extraction: 97% EXTRACTED · 3% INFERRED · 0% AMBIGUOUS · INFERRED: 75 edges (avg confidence: 0.85)
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

## God Nodes (most connected - your core abstractions)
1. `WalkBuilder` - 56 edges
2. `Ignore` - 35 edges
3. `builder()` - 34 edges
4. `one_matcher()` - 34 edges
5. `tmpdir()` - 34 edges
6. `Candidate` - 33 edges
7. `tmpdir()` - 33 edges
8. `matchedf()` - 33 edges
9. `Glob` - 32 edges
10. `DirEntry` - 32 edges

## Surprising Connections (you probably didn't know these)
- `Gitignore` --references--> `GlobSet`  [EXTRACTED]
  ignore/src/gitignore.rs → globset/src/lib.rs
- `Types` --references--> `GlobSet`  [EXTRACTED]
  ignore/src/types.rs → globset/src/lib.rs
- `GitignoreBuilder` --references--> `GlobSetBuilder`  [EXTRACTED]
  ignore/src/gitignore.rs → globset/src/lib.rs
- `max_filesize_respects_follow_links()` --calls--> `symlink()`  [INFERRED]
  ignore/src/incremental.rs → ignore/src/walk.rs
- `symlink_path_stays_under_lexical_root()` --calls--> `symlink()`  [INFERRED]
  ignore/src/incremental.rs → ignore/src/walk.rs

## Import Cycles
- 1-file cycle: `ignore/src/walk.rs -> ignore/src/walk.rs`
- 1-file cycle: `ignore/src/incremental.rs -> ignore/src/incremental.rs`
- 1-file cycle: `ignore/src/dir.rs -> ignore/src/dir.rs`
- 1-file cycle: `globset/src/glob.rs -> globset/src/glob.rs`
- 1-file cycle: `globset/src/lib.rs -> globset/src/lib.rs`
- 1-file cycle: `ignore/src/gitignore.rs -> ignore/src/gitignore.rs`
- 1-file cycle: `ignore/src/types.rs -> ignore/src/types.rs`
- 1-file cycle: `ignore/src/overrides.rs -> ignore/src/overrides.rs`
- 1-file cycle: `ignore/src/pathutil.rs -> ignore/src/pathutil.rs`
- 1-file cycle: `globset/src/pathutil.rs -> globset/src/pathutil.rs`
- 1-file cycle: `ignore/src/default_types.rs -> ignore/src/default_types.rs`
- 2-file cycle: `ignore/src/lib.rs -> ignore/src/walk.rs -> ignore/src/lib.rs`
- 2-file cycle: `ignore/src/incremental.rs -> ignore/src/walk.rs -> ignore/src/incremental.rs`
- 2-file cycle: `ignore/src/incremental.rs -> ignore/src/lib.rs -> ignore/src/incremental.rs`
- 2-file cycle: `globset/src/glob.rs -> globset/src/lib.rs -> globset/src/glob.rs`
- 3-file cycle: `ignore/src/gitignore.rs -> ignore/src/lib.rs -> ignore/src/walk.rs -> ignore/src/gitignore.rs`
- 3-file cycle: `ignore/src/dir.rs -> ignore/src/lib.rs -> ignore/src/walk.rs -> ignore/src/dir.rs`
- 3-file cycle: `ignore/src/dir.rs -> ignore/src/lib.rs -> ignore/src/incremental.rs -> ignore/src/dir.rs`
- 3-file cycle: `ignore/src/lib.rs -> ignore/src/walk.rs -> ignore/src/overrides.rs -> ignore/src/lib.rs`
- 3-file cycle: `ignore/src/incremental.rs -> ignore/src/overrides.rs -> ignore/src/lib.rs -> ignore/src/incremental.rs`

## Communities (34 total, 8 thin omitted)

### Community 0 - "Community 0"
Cohesion: 0.06
Nodes (66): assert_paths(), broken_symlink_follow_links_error(), broken_symlink_root_follow_links_error(), check_symlink_loop(), custom_ignore(), custom_ignore_exclusive_use(), device_num(), empty() (+58 more)

### Community 1 - "Community 1"
Cohesion: 0.07
Nodes (48): absolute_parent(), absolute_parent_anchored(), absolute_parent_matchers_are_cached_across_roots(), absolute_parent_repository_without_rules(), absolute_parent_rules_through_empty_directories(), create_gitignore(), custom_ignore(), custom_ignore_over_ignore() (+40 more)

### Community 2 - "Community 2"
Cohesion: 0.05
Nodes (22): DirEntry, DirEntryInner, Raw, Stdin, Walkdir, DirEntryRaw, FnVisitorImp<'s>, path_equals() (+14 more)

### Community 3 - "Community 3"
Cohesion: 0.10
Nodes (58): anchored_child_rule_uses_child_root(), builder(), builders(), cached_allowed_parent_matches_children(), cached_ignored_parent_matches_children(), cached_parent_matcher_is_used_for_directory(), CachedDir, Allowed (+50 more)

### Community 4 - "Community 4"
Cohesion: 0.09
Nodes (44): build_empty_gitignore(), build_python_gitignore(), build_single_directory_whitelist(), CountBuilder, CountBuilder<'s>, CountVisitor, dense_ignore_walk(), dense_ignore_workspace() (+36 more)

### Community 5 - "Community 5"
Cohesion: 0.07
Nodes (16): DEFAULT_TYPES, FileTypeDef, Glob, Glob<'a>, GlobInner, Matched, UnmatchedIgnore, matched! (+8 more)

### Community 6 - "Community 6"
Cohesion: 0.06
Nodes (18): DirEntry, X, Y, main(), Error, Glob, InvalidDefinition, Io (+10 more)

### Community 7 - "Community 7"
Cohesion: 0.07
Nodes (24): Match, Ignore, None, Match<T>, Whitelist, absolute_path(), allow_directories(), case_insensitive() (+16 more)

### Community 8 - "Community 8"
Cohesion: 0.08
Nodes (4): Glob, GlobSet, GlobSetVisitor, GlobVisitor

### Community 9 - "Community 9"
Cohesion: 0.14
Nodes (18): basename_literal_length_filter(), basename_regex_parent_component(), basename_regex_with_path_rules(), build_glob_set(), debug(), debug_impl_is_concise(), empty_set_works(), literal_length_filter() (+10 more)

### Community 10 - "Community 10"
Cohesion: 0.11
Nodes (10): Glob, MatchStrategy, BasenameLiteral, BasenameRegex, Extension, Literal, Prefix, Regex (+2 more)

### Community 11 - "Community 11"
Cohesion: 0.10
Nodes (14): Error, ErrorKind, DanglingEscape, InvalidRange, InvalidRecursive, NestedAlternates, Regex, TooManyNestedAlternates (+6 more)

### Community 12 - "Community 12"
Cohesion: 0.12
Nodes (15): gi_from_str(), ignored!, not_ignored!, parse_excludes_file1(), parse_excludes_file2(), parse_excludes_file4(), parse_excludes_file6(), path_string() (+7 more)

### Community 13 - "Community 13"
Cohesion: 0.11
Nodes (21): BSESC, CASEI, EALTRE, baseliteral!, basetokens!, ext!, extract!, literal! (+13 more)

### Community 14 - "Community 14"
Cohesion: 0.18
Nodes (5): Candidate, ExtensionStrategy, PatternSetPoolFn, RegexSetStrategy, RequiredExtensionStrategy

### Community 15 - "Community 15"
Cohesion: 0.16
Nodes (19): EXT, ext_glob(), EXT_PAT, ext_regex(), LONG, long_glob(), LONG_PAT, long_regex() (+11 more)

### Community 16 - "Community 16"
Cohesion: 0.13
Nodes (15): bytes_to_escaped_literal(), char_to_escaped_literal(), class(), classn(), rclass(), rclassn(), Token, Alternates (+7 more)

### Community 17 - "Community 17"
Cohesion: 0.16
Nodes (6): class_literal_separator(), GlobBuilder, GlobBuilder<'a>, GlobOptions, nested_alternates_depth_limit(), Parser

### Community 20 - "Community 20"
Cohesion: 0.18
Nodes (9): GlobSetMatchStrategy, BasenameLiteral, Extension, Literal, Prefix, RequiredExtension, Suffix, debug! (+1 more)

### Community 21 - "Community 21"
Cohesion: 0.16
Nodes (7): Candidate<'a>, file_name(), file_name_ext(), ext!, name!, normalize!, normalize_path()

### Community 22 - "Community 22"
Cohesion: 0.22
Nodes (4): ends_with(), GlobMatcher, GlobStrategic, starts_with()

### Community 23 - "Community 23"
Cohesion: 0.35
Nodes (5): case_insensitive(), empty_builder_preserves_root(), GitignoreBuilder, invalid_utf8_preserves_later_patterns(), test_issue_2745()

### Community 24 - "Community 24"
Cohesion: 0.26
Nodes (9): file_name(), file_name_basic(), file_name_dot_components(), file_name_trailing_dot(), is_file_name(), is_hidden_entry(), is_hidden_path_only(), strip_prefix() (+1 more)

### Community 28 - "Community 28"
Cohesion: 0.33
Nodes (9): excludes_file_default(), expand_tilde(), gitconfig_excludes_path(), gitconfig_global_env_contents(), gitconfig_home_contents(), gitconfig_system_contents(), gitconfig_xdg_contents(), home_dir() (+1 more)

### Community 29 - "Community 29"
Cohesion: 0.25
Nodes (4): Hasher, .OFFSET_BASIS, .PRIME, HashMap

### Community 30 - "Community 30"
Cohesion: 0.43
Nodes (7): get_gitignore(), IGNORE_FILE, test_dirs_in_deep(), test_dirs_in_root(), test_files_in_deep(), test_files_in_root(), test_path_should_be_under_root()

## Knowledge Gaps
- **47 isolated node(s):** `Quit`, `Work`, `Gitignore`, `Hidden`, `Override` (+42 more)
  These have ≤1 connection - possible missing edges. (Counts symbols only; 185 node(s) total have ≤1 connection when file, concept and rationale nodes are included.)
- **8 thin communities (<3 nodes) omitted from report** — run `graphify query` to explore isolated nodes.

## Suggested Questions
_Questions this graph is uniquely positioned to answer:_

- **Why does `Error` connect `Community 6` to `Community 0`, `Community 1`, `Community 3`, `Community 4`, `Community 5`, `Community 7`, `Community 8`, `Community 12`?**
  _High betweenness centrality (0.109) - this node is a cross-community bridge._
- **What connects `Quit`, `Work`, `Gitignore` to the rest of the system?**
  _47 weakly-connected nodes found - possible missing edges._
- **Should `Community 0` be split into smaller, more focused modules?**
  _Cohesion score 0.056807170542635656 - nodes in this community are weakly interconnected._
- **Why does `GlobSet` connect `Community 18` to `Community 5`, `Community 8`, `Community 9`, `Community 10`, `Community 11`, `Community 12`, `Community 15`, `Community 20`, `Community 27`?**
  _High betweenness centrality (0.093) - this node is a cross-community bridge._
- **Should `Community 1` be split into smaller, more focused modules?**
  _Cohesion score 0.07280927835051547 - nodes in this community are weakly interconnected._
- **Why does `Candidate` connect `Community 14` to `Community 9`, `Community 12`, `Community 13`, `Community 15`, `Community 18`, `Community 21`, `Community 22`, `Community 26`?**
  _High betweenness centrality (0.067) - this node is a cross-community bridge._
- **Should `Community 2` be split into smaller, more focused modules?**
  _Cohesion score 0.05266106442577031 - nodes in this community are weakly interconnected._