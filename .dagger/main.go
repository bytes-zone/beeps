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

func (m *Beeps) buildContainer(source *dagger.Directory) *dagger.Container {
	return dag.Container().
		From("rust:1.82.0").
		WithMountedCache("/usr/local/cargo/registry", dag.CacheVolume("cargo-registry")).
		WithMountedCache("/src/target", dag.CacheVolume("rust-compilation")).
		WithExec([]string{"rustup", "component", "add", "clippy"}).
		WithExec([]string{"cargo", "install", "typos-cli"}).
		WithMountedDirectory("/src", source).
		WithWorkdir("/src")
}

func (m *Beeps) All(
	ctx context.Context,
	// +optional
	// +defaultPath=.
	source *dagger.Directory,
) error {
	eg, ctx := errgroup.WithContext(ctx)

	eg.Go(func() error {
		_, err := m.Build(ctx, source, false).Sync(ctx)
		return err
	})

	eg.Go(func() error {
		_, err := m.Clippy(ctx, source).Sync(ctx)
		return err
	})

	eg.Go(func() error {
		_, err := m.Typos(ctx, source).Sync(ctx)
		return err
	})

	return eg.Wait()
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

	return m.buildContainer(source).WithExec(command)
}

// Lint source code with Clippy
func (m *Beeps) Clippy(ctx context.Context, source *dagger.Directory) *dagger.Container {
	return m.buildContainer(source).
		WithExec([]string{"cargo", "clippy"})
}

// Find typos with Typos
func (m *Beeps) Typos(ctx context.Context, source *dagger.Directory) *dagger.Container {
	return m.buildContainer(source).
		WithExec([]string{"typos"})
}

// Returns a container that echoes whatever string argument is provided
func (m *Beeps) ContainerEcho(stringArg string) *dagger.Container {
	return dag.Container().From("alpine:latest").WithExec([]string{"echo", stringArg})
}

// Returns lines that match a pattern in the files of the provided Directory
func (m *Beeps) GrepDir(ctx context.Context, directoryArg *dagger.Directory, pattern string) (string, error) {
	return dag.Container().
		From("alpine:latest").
		WithMountedDirectory("/mnt", directoryArg).
		WithWorkdir("/mnt").
		WithExec([]string{"grep", "-R", pattern, "."}).
		Stdout(ctx)
}
