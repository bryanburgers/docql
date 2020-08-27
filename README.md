Generate static HTML documentation for GraphQL APIs.


## Overview

[GraphiQL] is great. So are tools like [Altair] and [Insomnia]. But they aren't
necessarily enough.

`docql` comes in when you want documentation for GraphQL APIs that lives in a
shared place. Having HTML documentation allows teams to link to specific
objects and fields to enhance conversation, reference the docs when away from
the computer, and generally have a place to see the entire GraphQL schema at a
glance.

[GraphiQL]: https://github.com/graphql/graphiql
[Altair]: https://altair.sirmuel.design/
[Insomnia]: https://insomnia.rest/graphql/

## Examples

* [GitHub v4 API][github v4]: [generated][github v4 generated]
* [GraphQL's example Star Wars API][swapi]: [generated][swapi generated]

[github v4]: https://docs.github.com/en/graphql
[swapi]: https://swapi.graph.cool/
[github v4 generated]: https://bryanburgers.github.io/docql/github/
[swapi generated]: https://bryanburgers.github.io/docql/swapi/


## Use

There are two ways to use `docql`.

### npx

The easiest way to get started is to run `docql` off of the npm registry.

```
npx docql -e $API -o ./doc
```


### native binaries

If native binaries are more your style and you have access to [Rust]'s `cargo`,
you can install with `cargo install`.

```
cargo install docql
docql -e $API -o ./doc
```

[crates.io]: https://crates.io
[Rust]: https://rust-lang.org


## Command line options

```
USAGE:
    docql [OPTIONS] --output <path> <--endpoint <url>|--schema <path>>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --endpoint <url>        The URL of the GraphQL endpoint to document
    -x, --header <header>...    Additional headers when executing the GraphQL introspection query (e.g. `-x
                                "Authorization: Bearer abcdef"`
    -n, --name <name>           The name to give to the schema (used in the title of the page) [default: GraphQL Schema]
    -o, --output <path>         The directory to put the generated documentation
    -s, --schema <path>         The output of a GraphQL introspection query already stored locally
```
