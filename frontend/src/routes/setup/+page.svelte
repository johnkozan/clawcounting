<script lang="ts">
	import { goto } from '$app/navigation';
	import { auth as authApi, ApiError } from '$lib/api';
	import { authStore } from '$lib/stores/auth';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import * as Card from '$lib/components/ui/card';
	import { Landmark } from '@lucide/svelte';
	import { onMount } from 'svelte';

	let name = $state('');
	let email = $state('');
	let password = $state('');
	let confirmPassword = $state('');
	let error = $state('');
	let loading = $state(false);
	let checking = $state(true);

	onMount(async () => {
		try {
			const status = await authApi.setupStatus();
			if (!status.needs_setup) {
				goto('/login');
				return;
			}
		} catch {
			// If the endpoint fails, redirect to login
			goto('/login');
			return;
		}
		checking = false;
	});

	async function handleSubmit(e: Event) {
		e.preventDefault();
		error = '';

		if (password !== confirmPassword) {
			error = 'Passwords do not match';
			return;
		}

		if (password.length < 8) {
			error = 'Password must be at least 8 characters';
			return;
		}

		loading = true;
		try {
			const res = await authApi.setup(name, email, password);
			localStorage.setItem('token', res.access_token);
			localStorage.setItem('refresh_token', res.refresh_token);
			await authStore.initialize();
			goto('/dashboard');
		} catch (err) {
			if (err instanceof ApiError) {
				error = err.message;
			} else {
				error = 'An unexpected error occurred';
			}
		} finally {
			loading = false;
		}
	}
</script>

<svelte:head>
	<title>Setup - ClawCounting</title>
</svelte:head>

{#if checking}
	<div class="flex min-h-screen items-center justify-center bg-background">
		<div class="text-muted-foreground">Loading...</div>
	</div>
{:else}
	<div class="flex min-h-screen items-center justify-center bg-background px-4">
		<Card.Root class="w-full max-w-sm">
			<Card.Header class="text-center">
				<div class="mx-auto mb-2 flex h-12 w-12 items-center justify-center rounded-lg bg-primary">
					<Landmark class="h-6 w-6 text-primary-foreground" />
				</div>
				<Card.Title class="text-2xl">Welcome to ClawCounting</Card.Title>
				<Card.Description>Create your admin account to get started.</Card.Description>
			</Card.Header>
			<Card.Content>
				<form onsubmit={handleSubmit} class="space-y-4">
					{#if error}
						<div class="rounded-md bg-destructive/10 px-3 py-2 text-sm text-destructive">
							{error}
						</div>
					{/if}
					<div class="space-y-2">
						<Label for="name">Name</Label>
						<Input
							id="name"
							type="text"
							placeholder="Admin"
							bind:value={name}
							required
							autocomplete="name"
						/>
					</div>
					<div class="space-y-2">
						<Label for="email">Email</Label>
						<Input
							id="email"
							type="email"
							placeholder="admin@example.com"
							bind:value={email}
							required
							autocomplete="email"
						/>
					</div>
					<div class="space-y-2">
						<Label for="password">Password</Label>
						<Input
							id="password"
							type="password"
							bind:value={password}
							required
							autocomplete="new-password"
						/>
					</div>
					<div class="space-y-2">
						<Label for="confirm-password">Confirm Password</Label>
						<Input
							id="confirm-password"
							type="password"
							bind:value={confirmPassword}
							required
							autocomplete="new-password"
						/>
					</div>
					<Button type="submit" class="w-full" disabled={loading}>
						{loading ? 'Creating account...' : 'Create Account'}
					</Button>
				</form>
			</Card.Content>
		</Card.Root>
	</div>
{/if}
