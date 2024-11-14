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

func (m *Beeps) buildContainer(source *dagger.Directory, cachePrefix string) *dagger.Container {
	return dag.Container().
		From("rust:1.82.0").
		WithMountedCache("/usr/local/cargo/registry", dag.CacheVolume("cargo-registry")).
		WithMountedCache("/src/target", dag.CacheVolume(fmt.Sprintf("rust-compilation-%s", cachePrefix))).
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

	eg.Go(func() error {
		_, err := m.Fmt(ctx, source).Sync(ctx)
		return err
	})

	eg.Go(func() error {
		_, err := m.Machete(ctx, source).Sync(ctx)
		return err
	})

	eg.Go(func() error {
		_, err := m.Test(ctx, source).Sync(ctx)
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

	return m.buildContainer(source, "build").
		WithMountedDirectory("/src", source).
		WithWorkdir("/src").
		WithExec(command)
}

// Run unit and integration tests for the project
func (m *Beeps) Test(
	ctx context.Context,
	// +defaultPath=.
	source *dagger.Directory,
) *dagger.Container {
	return m.buildContainer(source, "test").
		WithExec([]string{"cargo", "install", "sqlx-cli", "--features", "postgres"}).
		WithServiceBinding(
			"postgres",
			m.Db(
				ctx,
				dag.SetSecret("pguser", "beeps"),
				dag.SetSecret("pgpassword", "beeps"),
			).AsService(),
		).
		WithEnvVariable("DATABASE_URL", "postgres://beeps:beeps@postgres:5432/beeps").
		WithMountedDirectory("/src", source).
		WithWorkdir("/src").
		WithExec([]string{"cargo", "sqlx", "migrate", "run", "--source", "beeps-server/migrations"}).
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
func (m *Beeps) Clippy(ctx context.Context, source *dagger.Directory) *dagger.Container {
	return m.buildContainer(source, "clippy").
		WithExec([]string{"rustup", "component", "add", "clippy"}).
		WithMountedDirectory("/src", source).
		WithWorkdir("/src").
		WithExec([]string{"cargo", "clippy"})
}

// Find typos with Typos
func (m *Beeps) Typos(ctx context.Context, source *dagger.Directory) *dagger.Container {
	return m.buildContainer(source, "typos").
		WithExec([]string{"cargo", "install", "typos-cli"}).
		WithMountedDirectory("/src", source).
		WithWorkdir("/src").
		WithExec([]string{"typos"})
}

// Lint source code with `cargo fmt`
func (m *Beeps) Fmt(ctx context.Context, source *dagger.Directory) *dagger.Container {
	return m.buildContainer(source, "fmt").
		WithExec([]string{"rustup", "component", "add", "rustfmt"}).
		WithMountedDirectory("/src", source).
		WithWorkdir("/src").
		WithExec([]string{"cargo", "fmt", "--check"})
}

// Lint source code with `cargo machete`
func (m *Beeps) Machete(ctx context.Context, source *dagger.Directory) *dagger.Container {
	return m.buildContainer(source, "fmt").
		WithExec([]string{"cargo", "install", "cargo-machete"}).
		WithMountedDirectory("/src", source).
		WithWorkdir("/src").
		WithExec([]string{"cargo", "machete"})
}
