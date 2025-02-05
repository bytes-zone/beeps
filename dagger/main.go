package main

import (
	"context"
	"dagger/beeps/internal/dagger"
	"fmt"
	"strings"

	"golang.org/x/sync/errgroup"
)

type Beeps struct{}

const RUST_CONTAINER_IMAGE = "rust:1.84.1"

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
	container string
	wasmBuild string
	wasmSize  string
}

func section(title string, body string) string {
	return fmt.Sprintf("## %s\n\n```\n%s\n```", title, body)
}

func (n *NiceOutput) Format() string {
	arr := []string{
		section("Container", n.container),
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
		out, err := m.TestServerContainerImage(ctx, source).Stdout(ctx)
		nice.container = out
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
		WithExec([]string{"/bin/bash", "-c", "apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*"}).
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

// Test the server container image
func (m *Beeps) TestServerContainerImage(
	ctx context.Context,
	// +optional
	// +defaultPath=.
	// +ignore=["target", ".git", ".dagger", "pgdata"]
	source *dagger.Directory,
) *dagger.Container {
	return m.ServerContainerImage(ctx, source).
		WithExec([]string{"/bin/beeps-server", "--version"})
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
