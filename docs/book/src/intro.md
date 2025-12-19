# Introduction

Medley provides a W3C-style EBNF grammar macro (`grammar!`), a zero-copy streaming parser, and an optional AST builder. It targets small dependency footprints and predictable performance.

This book explains how to define grammars, parse streams, build ASTs, and handle errors. The examples mirror the code in the repository so you can copy/paste and run them with `cargo run --example <name>`.
