# unused-export-lint

A small tool that finds imports and exports in typed Javascript files with tree-sitter.

## TODOs

It's goal is to provide a CLI interface, to check wich exports are unused and a LSP to integrate into your editor of choice.

- filter the current hashmap to finde unused exports
- create a LSP interface
- find some way to cache results of files that did not change
