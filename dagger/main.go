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
}

func section(title string, body string) string {
	return fmt.Sprintf("## %s\n\n```\n%s\n```", title, body)
}

func (n *NiceOutput) Format() string {
	arr := []string{
		section("Container", n.container),
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
