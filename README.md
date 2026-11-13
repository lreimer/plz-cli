# Copilot, for your terminal

A CLI tool that generates shell scripts from a human readable description.

## Installation

You can install `plz` by running the following command in your terminal.

```
curl -fsSL https://raw.githubusercontent.com/m1guelpf/plz-cli/main/install.sh | sh -
```

### Homebrew

You can also install `plz` using [Homebrew](https://brew.sh/).

```sh
$ brew install plz-cli
```

You may need to close and reopen your terminal after installation. Alternatively, you can download the binary corresponding to your OS from the [latest release](https://github.com/m1guelpf/plz-cli/releases/latest).

## Usage

Set `PLZ_PROVIDER` to `openai` (the default) or `google` (`gemini` is also accepted).
Provider names are case-insensitive. An unset or blank value selects OpenAI.

### OpenAI

`plz` uses OpenAI's [GPT-5.4 mini](https://developers.openai.com/api/docs/models/gpt-5.4-mini) model by default. To use it, you'll need an API key from [the OpenAI API dashboard](https://platform.openai.com/api-keys), saved as `OPENAI_API_KEY` (you can also save it in your bash/zsh profile for persistence between sessions).

```bash
export OPENAI_API_KEY='sk-XXXXXXXX'
```

To use a different model, set `OPENAI_MODEL` (you can also save it in your bash/zsh profile):

```bash
export OPENAI_MODEL='gpt-4.1-mini'
```

If `OPENAI_MODEL` is unset, empty, or whitespace-only, `plz` uses `gpt-5.4-mini`. Leading and trailing whitespace is ignored. The selected model must support the existing Chat Completions endpoint and request parameters.

Set `OPENAI_API_BASE` to override the default API base URL, `https://api.openai.com/v1`.

### Google Gemini

Get an API key from [Google AI Studio](https://aistudio.google.com/apikey), then configure:

```bash
export PLZ_PROVIDER='google'
export GEMINI_API_KEY='your-api-key'
export GEMINI_MODEL='gemini-2.5-flash'
```

`GEMINI_MODEL` is optional and defaults to `gemini-2.5-flash`. `GOOGLE_API_KEY`
is accepted as a fallback when `GEMINI_API_KEY` is unset or blank. No OpenAI key is
required when using Google.

Set `GEMINI_API_BASE` to override the default API base URL,
`https://generativelanguage.googleapis.com/v1beta`. Google uses the native
`models/{model}:generateContent` endpoint; model names must support that endpoint.
Model and API base settings ignore surrounding whitespace and use their defaults
when unset or blank. Only the selected provider's settings are used.

### Generating commands

Once you have configured your environment, run `plz` followed by whatever it is that you want to do (`plz show me all options for the plz cli`).

To get a full overview of all available options, run `plz --help`

```sh
$ plz --help
Generates bash scripts from the command line

Usage: plz [OPTIONS] <PROMPT>

Arguments:
  <PROMPT>  Description of the command to execute

Options:
  -y, --force    Run the generated program without asking for confirmation
  -h, --help     Print help information
  -V, --version  Print version information
```

## Develop

Make sure you have the latest version of rust installed (use [rustup](https://rustup.rs/)). Then, you can build the project by running `cargo build`, and run it with `cargo run`.

## License

This project is open-sourced under the MIT license. See [the License file](LICENSE) for more information.
