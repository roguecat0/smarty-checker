# smarty-checker

Lint Smarty templates.

## Usage

```sh
smarty-checker [-c config.toml] <file.tpl> [file.tpl ...]
```

Without a config file, the default rule set is used.

## Config

Rules can be enabled or disabled with a TOML `[rules]` table. Rule ids must be quoted because they contain `/`.

```toml
[rules]
"smarty/balanced-html-in-control-flow" = true
"smarty/no-control-flow-in-script" = true
"smarty/no-smarty-in-script" = false
```

## Rules

| Rule | Default | Description |
| --- | --- | --- |
| `smarty/balanced-html-in-control-flow` | `true` | Checks that direct HTML inside each Smarty control-flow branch is balanced. |
| `smarty/no-control-flow-in-script` | `true` | Reports Smarty control-flow blocks inside `<script>` bodies. |
| `smarty/no-smarty-in-script` | `false` | Reports any Smarty statement inside `<script>` bodies, including inline output. |
