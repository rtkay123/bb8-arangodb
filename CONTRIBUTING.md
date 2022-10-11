# Contributing

Contributions are welcome, and they are greatly appreciated! Every little
helps, and credit will always be given. You can contribute in many ways.

## Types of Contributions

### Report Bugs

Report bugs at <https://github.com/gabor-boros/bb8-arangodb/issues>.

If you are reporting a bug, please use the bug report template, and include:

- your operating system name and version
- any details about your local setup that might be helpful in troubleshooting
- detailed steps to reproduce the bug

### Fix Bugs

Look through the GitHub issues for bugs. Anything tagged with "bug" and
"help wanted" is open to whoever wants to implement it.

### Implement Features

Look through the GitHub issues for features. Anything tagged with "enhancement"
and "help wanted" is open to whoever wants to implement it. In case you added a
new source or target, do not forget to add them to the docs as well.

### Write Documentation

bb8-arangodb could always use more documentation, whether as part of the docs,
in docstrings, or even on the web in blog posts, articles, and such.

### Submit Feedback

The best way to send feedback is to file an [issue].

If you are proposing a feature:

- explain in detail how it would work
- keep the scope as narrow as possible, to make it easier to implement
- remember that this is a volunteer-driven project, and that contributions are
  welcome :)

[issue]: https://github.com/gabor-boros/bb8-arangodb/issues

## Get Started!

Ready to contribute? Here's how to set up `bb8-arangodb` for local development.

1. Fork the repository
2. Clone your fork locally

```shell
$ git clone git@github.com:your_name_here/bb8-arangodb.git
```

3. Create a branch for local development

```shell
$ git checkout -b github-username/bugfix-or-feature-name
```

4. When you're done making changes, check that your changes are formatted, passing linters, and tests are succeeding

```shell
$ cargo fmt
$ cargo clippy
$ cargo test
```

5. Commit your changes and push your branch to GitHub

We use [conventional commits](https://www.conventionalcommits.org/en/v1.0.0/), and we require every commit to
follow this pattern.

```shell
$ git add .
$ git commit -m "action(scope): summary"
$ git push origin github-username/bugfix-or-feature-name
```

6. Submit a pull request on GitHub

## Pull Request Guidelines

Before you submit a pull request, check that it meets these guidelines:

1. The pull request should include tests if applicable
2. Tests should pass for the PR

## Releasing new versions

1. Double-check past commits that nothing missed
1. Run git cliff --tag v<x.x.x> --unreleased --prepend CHANGELOG.md, where
   <x.x.x> is the upcoming release version
2. Validate changes in CHANGELOG.md
3. Commit the changes using git commit -sm "chore(changelog): update changelog"
4. Run git tag v<x.x.x> -sm "chore(release): cut release v<x.x.x>", where
   <x.x.x> is the upcoming release version
5. Run git push origin main and git push v<x.x.x>, where <x.x.x> is the new
   release version
