// A generated module for Beeps functions
//
// This module has been generated via dagger init and serves as a reference to
// basic module structure as you get started with Dagger.
//
// Two functions have been pre-created. You can modify, delete, or add to them,
// as needed. They demonstrate usage of arguments and return types using simple
// echo and grep commands. The functions can be called from the dagger CLI or
// from one of the SDKs.
//
// The first line in this comment block is a short description line and the
// rest is a long description with more detail on the module's purpose or usage,
// if appropriate. All modules should have a short description.

package main

import (
	"context"
	"dagger/beeps/internal/dagger"
	"fmt"
	"strings"

	"golang.org/x/sync/errgroup"
)

type Beeps struct{}

// Start a postgres server
func (m *Beeps) Postgres(init *dagger.File) *dagger.Container {
	return dag.Postgres(
		dag.SetSecret("postgres-user", "beeps"),
		dag.SetSecret("postgres-password", "beeps"),
		dagger.PostgresOpts{
			DbPort:     5432,
			DbName:     "beeps",
			Cache:      true,
			InitScript: dag.Directory().WithFile("init.sql", init),
		},
	).Database()
}

const RUST_CONTAINER_IMAGE = "rust:1.83.0"

func (m *Beeps) rustBase(cacheKey string) *dagger.Container {
	return dag.Container().
		From(RUST_CONTAINER_IMAGE).
		WithMountedCache("/root/.cargo", dag.CacheVolume(fmt.Sprintf("cargo-home-%s", cacheKey))).
		WithEnvVariable("CARGO_HOME", "/root/.cargo").
		WithMountedCache("/target", dag.CacheVolume(fmt.Sprintf("rust-compilation-%s", cacheKey))).
		WithEnvVariable("CARGO_TARGET_DIR", "/target").
		WithEnvVariable("PATH", "/root/.cargo/bin:/usr/local/cargo/bin:/usr/local/sbin:/usr/local/bin:/usr/sbin:/usr/bin:/sbin:/bin")
}

func cargoInstall(installFlags []string) dagger.WithContainerFunc {
	return func(c *dagger.Container) *dagger.Container {
		return c.WithExec(append([]string{"cargo", "install"}, installFlags...))
	}
}

func rustupComponent(component string) dagger.WithContainerFunc {
	return func(c *dagger.Container) *dagger.Container {
		return c.WithExec([]string{"rustup", "component", "add", component})
	}
}

func userSource(source *dagger.Directory) dagger.WithContainerFunc {
	return func(c *dagger.Container) *dagger.Container {
		return c.
			WithMountedDirectory("/src", source).
			WithWorkdir("/src")
	}
}

type NiceOutput struct {
	build     string
	test      string
	clippy    string
	typos     string
	fmt       string
	machete   string
	wasmBuild bool
}

func section(title string, body string) string {
	return fmt.Sprintf("## %s\n\n```\n%s\n```", title, body)
}

func (n *NiceOutput) Format() string {
	arr := []string{
		section("Build", n.build),
		section("Test", n.test),
		section("Clippy", n.clippy),
		section("Typos", n.typos),
		section("Fmt", n.fmt),
		section("Machete", n.machete),
		section("WASM Build", fmt.Sprintf("Success: %t", n.wasmBuild)),
	}
	return strings.Join(arr, "\n\n")
}

func (m *Beeps) All(
	ctx context.Context,
	// +optional
	// +defaultPath=.
	source *dagger.Directory,
) (string, error) {
	eg, ctx := errgroup.WithContext(ctx)

	nice := NiceOutput{}

	eg.Go(func() error {
		out, err := m.Build(ctx, source, false).Stderr(ctx)
		nice.build = out
		return err
	})

	eg.Go(func() error {
		out, err := m.Clippy(ctx, source).Stderr(ctx)
		nice.clippy = out
		return err
	})

	eg.Go(func() error {
		out, err := m.Typos(ctx, source).Stdout(ctx)
		nice.typos = out
		return err
	})

	eg.Go(func() error {
		out, err := m.Fmt(ctx, source).Stderr(ctx)
		nice.fmt = out
		return err
	})

	eg.Go(func() error {
		out, err := m.Machete(ctx, source).Stdout(ctx)
		nice.machete = out
		return err
	})

	eg.Go(func() error {
		out, err := m.Test(ctx, source).Stdout(ctx)
		nice.test = out
		return err
	})

	eg.Go(func() error {
		m.WasmBuild(ctx, source, "browser", "bundler")
		nice.wasmBuild = true
		return nil
	})

	err := eg.Wait()

	return nice.Format(), err
}

// Build beeps and beeps-server
func (m *Beeps) Build(
	ctx context.Context,
	// +optional
	// +defaultPath=.
	source *dagger.Directory,
	// +optional
	release bool,
) *dagger.Container {
	command := []string{"cargo", "build"}
	if release {
		command = append(command, "--release")
	}

	return m.rustBase("build").
		With(userSource(source)).
		WithExec(command)
}

// Run unit and integration tests for the project
func (m *Beeps) Test(
	ctx context.Context,
	// +defaultPath=.
	source *dagger.Directory,
) *dagger.Container {
	return m.rustBase("test").
		With(userSource(source)).
		WithExec([]string{"cargo", "test"})
}

func (m *Beeps) Db(
	ctx context.Context,
	user *dagger.Secret,
	password *dagger.Secret,
) *dagger.Container {
	return dag.Postgres(
		user,
		password,
		dagger.PostgresOpts{DbName: "beeps"},
	).Database()
}

// Lint source code with Clippy
func (m *Beeps) Clippy(
	ctx context.Context,
	// +defaultPath=.
	source *dagger.Directory,
) *dagger.Container {
	return m.rustBase("clippy").
		WithExec([]string{"rustup", "component", "add", "clippy"}).
		With(userSource(source)).
		WithExec([]string{"cargo", "clippy", "--", "--deny=warnings"})
}

// Find typos with Typos
func (m *Beeps) Typos(
	ctx context.Context,
	// +defaultPath=.
	source *dagger.Directory,
) *dagger.Container {
	return m.rustBase("typos").
		WithExec([]string{"cargo", "install", "typos-cli"}).
		With(userSource(source)).
		WithExec([]string{"typos"})
}

// Lint source code with `cargo fmt`
func (m *Beeps) Fmt(
	ctx context.Context,
	// +defaultPath=.
	source *dagger.Directory,
) *dagger.Container {
	return m.rustBase("fmt").
		WithExec([]string{"rustup", "component", "add", "rustfmt"}).
		With(userSource(source)).
		WithExec([]string{"cargo", "fmt", "--check"})
}

// Lint source code with `cargo machete`
func (m *Beeps) Machete(
	ctx context.Context,
	// +defaultPath=.
	source *dagger.Directory,
) *dagger.Container {
	return m.rustBase("machete").
		WithExec([]string{"cargo", "install", "cargo-machete"}).
		With(userSource(source)).
		WithExec([]string{"cargo", "machete"})
}

// Build WASM package
func (m *Beeps) WasmBuild(
	ctx context.Context,
	// +defaultPath=.
	source *dagger.Directory,
	// +default="browser"
	crate string,
	// +default="bundler"
	target string,
) *dagger.Directory {
	return m.rustBase("wasm-pack").
		WithExec([]string{"cargo", "install", "wasm-pack"}).
		With(userSource(source)).
		WithExec([]string{"wasm-pack", "build", crate, "--out-dir=/wasm-pack-out"}).
		Directory("/wasm-pack-out")
}
