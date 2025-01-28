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
func (m *Beeps) Postgres() *dagger.Service {
	return dag.Postgres(dagger.PostgresOpts{
		Version:  "17.2",
		User:     dag.SetSecret("postgres-user", "beeps"),
		Password: dag.SetSecret("postgres-password", "beeps"),
	}).Service()
}

const RUST_CONTAINER_IMAGE = "rust:1.84.0"

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
	wasmBuild string
	wasmSize  string
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
		section("WASM Build", n.wasmBuild),
		section("WASM Size", n.wasmSize),
	}
	return strings.Join(arr, "\n\n")
}

func (m *Beeps) All(
	ctx context.Context,
	// +optional
	// +defaultPath=.
	// +ignore=["target", ".git", ".dagger", "pgdata"]
	source *dagger.Directory,
) (string, error) {
	eg, ctx := errgroup.WithContext(ctx)

	nice := NiceOutput{}

	eg.Go(func() error {
		out, err := m.Clippy(ctx, source, true).Stderr(ctx)
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
		out, err := m.WasmBuild(ctx, source, "browser", "bundler").Stderr(ctx)
		nice.wasmBuild = out
		return err
	})

	eg.Go(func() error {
		out, err := m.WasmSize(ctx, source, "browser", "bundler")
		nice.wasmSize = out
		return err
	})

	err := eg.Wait()

	return nice.Format(), err
}

// Build beeps and beeps-server
func (m *Beeps) Build(
	ctx context.Context,
	// +optional
	// +defaultPath=.
	// +ignore=["target", ".git", ".dagger", "pgdata"]
	source *dagger.Directory,
	// +optional
	release bool,
	// +optional
	binary string,
) *dagger.Container {
	command := []string{"cargo", "build"}
	if release {
		command = append(command, "--release")
	}

	if binary != "" {
		command = append(command, "--bin", binary)
	}

	return m.rustBase("build").
		With(userSource(source)).
		WithExec(command)
}

// Build the server container image
func (m *Beeps) ServerContainerImage(
	ctx context.Context,
	// +optional
	// +defaultPath=.
	// +ignore=["target", ".git", ".dagger", "pgdata"]
	source *dagger.Directory,
) *dagger.Container {
	return dag.Container().
		From("bitnami/minideb:bookworm").
		WithFile(
			"/bin/beeps-server",
			m.Build(ctx, source, true, "beeps-server").
				WithExec([]string{"cp", "/target/release/beeps-server", "/beeps-server"}).
				File("/beeps-server"),
		).
		WithEntrypoint([]string{"/bin/beeps-server"}).
		WithLabel("org.opencontainers.image.description", "the Beeps server").
		WithExposedPort(3000)
}

// Run unit and integration tests for the project
func (m *Beeps) Test(
	ctx context.Context,
	// +defaultPath=.
	// +ignore=["target", ".git", ".dagger", "pgdata"]
	source *dagger.Directory,
) *dagger.Container {
	return m.rustBase("test").
		WithExec([]string{"cargo", "install", "sqlx-cli", "--no-default-features", "--features=postgres"}).

		// Database
		WithServiceBinding("postgres", m.Postgres()).
		WithEnvVariable("DATABASE_URL", "postgres://beeps:beeps@postgres:5432/beeps").

		// Test
		With(userSource(source)).
		WithExec([]string{"sqlx", "migrate", "run", "--source", "beeps-server/migrations"}).
		WithExec([]string{"cargo", "test"})
}

// Lint source code with Clippy
func (m *Beeps) Clippy(
	ctx context.Context,
	// +defaultPath=.
	// +ignore=["target", ".git", ".dagger", "pgdata"]
	source *dagger.Directory,
	// +optional
	noDeps bool,
) *dagger.Container {
	command := []string{"cargo", "clippy", "--", "--deny=warnings"}
	if noDeps {
		command = append(command, "--no-deps")
	}

	return m.rustBase("clippy").
		WithExec([]string{"rustup", "component", "add", "clippy"}).
		With(userSource(source)).
		WithExec(command)
}

const TYPOS_VERSION = "1.29.4"

// Find typos with Typos
func (m *Beeps) Typos(
	ctx context.Context,
	// +defaultPath=.
	// +ignore=["target", ".git", ".dagger", "pgdata"]
	source *dagger.Directory,
) *dagger.Container {
	release := dag.HTTP(fmt.Sprintf(
		"https://github.com/crate-ci/typos/releases/download/v%s/typos-v%s-x86_64-unknown-linux-musl.tar.gz",
		TYPOS_VERSION,
		TYPOS_VERSION,
	))

	return dag.Container().
		From("alpine:3.21.2").
		WithFile("release.tgz", release).
		WithExec([]string{"tar", "-xzf", "release.tgz"}).
		WithExec([]string{"mv", "typos", "/bin/typos"}).
		// done installing typos, now we can check!
		With(userSource(source)).
		WithExec([]string{"typos"})
}

// Lint source code with `cargo fmt`
func (m *Beeps) Fmt(
	ctx context.Context,
	// +defaultPath=.
	// +ignore=["target", ".git", ".dagger", "pgdata"]
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
	// +ignore=["target", ".git", ".dagger", "pgdata"]
	source *dagger.Directory,
) *dagger.Container {
	return dag.Container().
		From("ghcr.io/bnjbvr/cargo-machete:v0.7.0").
		With(userSource(source)).
		WithExec([]string{}, dagger.ContainerWithExecOpts{UseEntrypoint: true})
}

const WASM_PACK_VERSION = "0.13.1"

const WASM_BINDGEN_VERSION = "0.2.100"

// Build WASM package
func (m *Beeps) WasmBuild(
	ctx context.Context,
	// +defaultPath=.
	// +ignore=["target", ".git", ".dagger", "pgdata"]
	source *dagger.Directory,
	// +default="browser"
	crate string,
	// +default="bundler"
	target string,
) *dagger.Container {
	release := dag.HTTP(fmt.Sprintf(
		"https://github.com/rustwasm/wasm-pack/releases/download/v%s/wasm-pack-v%s-x86_64-unknown-linux-musl.tar.gz",
		WASM_PACK_VERSION,
		WASM_PACK_VERSION,
	))

	return m.rustBase("wasm-pack").
		WithExec([]string{"rustup", "component", "add", "rust-std", "--target", "wasm32-unknown-unknown"}).

		// install wasm-pack
		WithFile("release.tgz", release).
		WithExec([]string{"tar", "-xz", "--strip-components=1", "--file=release.tgz"}).
		WithExec([]string{"mv", "wasm-pack", "/bin"}).

		// build the WASM package
		With(userSource(source)).
		WithExec([]string{"wasm-pack", "build", crate, "--out-dir=/pkg"})
}

// Check WASM sizes
func (m *Beeps) WasmSize(
	ctx context.Context,
	// +defaultPath=.
	// +ignore=["target", ".git", ".dagger", "pgdata"]
	source *dagger.Directory,
	// +default="browser"
	crate string,
	// +default="bundler"
	target string,
) (string, error) {
	return m.WasmBuild(ctx, source, crate, target).
		WithExec([]string{"bash", "-c", "for target in /pkg/*.js /pkg/*.wasm; do gzip -9c $target > $target.gz; done"}).
		WithExec([]string{"ls", "-lh", "/pkg"}).
		Stdout(ctx)
}
