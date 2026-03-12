<script lang="ts">
	import { onMount } from 'svelte';
	import {
		users,
		type User,
		type CreateUserRequest,
		type CreateServiceAccountRequest,
		ApiError
	} from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import * as Tabs from '$lib/components/ui/tabs';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft, Plus, Copy } from '@lucide/svelte';

	let items = $state<User[]>([]);
	let loading = $state(true);

	// Create user dialog
	let createDialogOpen = $state(false);
	let creating = $state(false);
	let createTab = $state('human');

	// Human user form
	let humanForm = $state<CreateUserRequest>({
		name: '',
		email: '',
		password: ''
	});

	// Service account form
	let serviceForm = $state<CreateServiceAccountRequest>({
		name: ''
	});

	// API key display
	let showApiKey = $state('');

	onMount(async () => {
		try {
			const res = await users.list({ limit: 200 });
			items = res.data;
		} catch {
			toast.error('Failed to load users');
		} finally {
			loading = false;
		}
	});

	async function createHumanUser(e: Event) {
		e.preventDefault();
		creating = true;
		try {
			await users.create(humanForm);
			toast.success('User created');
			createDialogOpen = false;
			humanForm = { name: '', email: '', password: '' };
			const res = await users.list({ limit: 200 });
			items = res.data;
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to create user');
		} finally {
			creating = false;
		}
	}

	async function createServiceAccount(e: Event) {
		e.preventDefault();
		creating = true;
		try {
			const res = await users.createServiceAccount(serviceForm);
			showApiKey = res.data.api_key;
			toast.success('Service account created');
			serviceForm = { name: '' };
			const listRes = await users.list({ limit: 200 });
			items = listRes.data;
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to create service account');
		} finally {
			creating = false;
		}
	}

	async function toggleActive(user: User) {
		try {
			const res = await users.update(user.id, { is_active: !user.is_active });
			const idx = items.findIndex((u) => u.id === user.id);
			if (idx >= 0) items[idx] = res.data;
			toast.success(res.data.is_active ? 'User activated' : 'User deactivated');
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
		}
	}

	function copyApiKey() {
		navigator.clipboard.writeText(showApiKey);
		toast.success('API key copied');
	}
</script>

<svelte:head>
	<title>Users - ClawCounting</title>
</svelte:head>

<div class="mx-auto max-w-4xl space-y-6">
	<div class="flex items-center gap-4">
		<a href="/settings" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">Users</h1>
		<div class="ml-auto">
			<Button onclick={() => (createDialogOpen = true)}>
				<Plus class="mr-2 h-4 w-4" />
				New User
			</Button>
		</div>
	</div>

	<Card.Root>
		<Card.Content class="p-0">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Name</Table.Head>
						<Table.Head>Email</Table.Head>
						<Table.Head>Type</Table.Head>
						<Table.Head>Status</Table.Head>
						<Table.Head>Created</Table.Head>
						<Table.Head></Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#if loading}
						<Table.Row>
							<Table.Cell colspan={6} class="text-center text-muted-foreground">
								Loading...
							</Table.Cell>
						</Table.Row>
					{:else if items.length === 0}
						<Table.Row>
							<Table.Cell colspan={6} class="text-center text-muted-foreground">
								No users yet.
							</Table.Cell>
						</Table.Row>
					{:else}
						{#each items as user}
							<Table.Row>
								<Table.Cell class="font-medium">{user.name}</Table.Cell>
								<Table.Cell class="text-sm text-muted-foreground">{user.email ?? '-'}</Table.Cell>
								<Table.Cell>
									<Badge variant={user.user_type === 'human' ? 'default' : 'secondary'}>
										{user.user_type}
									</Badge>
								</Table.Cell>
								<Table.Cell>
									<Badge variant={user.is_active ? 'secondary' : 'destructive'}>
										{user.is_active ? 'Active' : 'Inactive'}
									</Badge>
								</Table.Cell>
								<Table.Cell class="text-sm text-muted-foreground">
									{user.created_at.split('T')[0]}
								</Table.Cell>
								<Table.Cell>
									<Button variant="ghost" size="sm" onclick={() => toggleActive(user)}>
										{user.is_active ? 'Deactivate' : 'Activate'}
									</Button>
								</Table.Cell>
							</Table.Row>
						{/each}
					{/if}
				</Table.Body>
			</Table.Root>
		</Card.Content>
	</Card.Root>
</div>

<!-- Create dialog -->
<Dialog.Root bind:open={createDialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>New User</Dialog.Title>
		</Dialog.Header>
		<Tabs.Root value={createTab} onValueChange={(v) => { if (v) createTab = v; }}>
			<Tabs.List class="grid w-full grid-cols-2">
				<Tabs.Trigger value="human">Human User</Tabs.Trigger>
				<Tabs.Trigger value="service">Service Account</Tabs.Trigger>
			</Tabs.List>
			<Tabs.Content value="human">
				<form onsubmit={createHumanUser} class="space-y-4 pt-4">
					<div class="space-y-2">
						<Label for="humanName">Name</Label>
						<Input id="humanName" bind:value={humanForm.name} required />
					</div>
					<div class="space-y-2">
						<Label for="humanEmail">Email</Label>
						<Input id="humanEmail" type="email" bind:value={humanForm.email} required />
					</div>
					<div class="space-y-2">
						<Label for="humanPassword">Password</Label>
						<Input id="humanPassword" type="password" bind:value={humanForm.password} required />
					</div>
					<Button type="submit" class="w-full" disabled={creating}>
						{creating ? 'Creating...' : 'Create User'}
					</Button>
				</form>
			</Tabs.Content>
			<Tabs.Content value="service">
				<form onsubmit={createServiceAccount} class="space-y-4 pt-4">
					<div class="space-y-2">
						<Label for="serviceName">Name</Label>
						<Input id="serviceName" bind:value={serviceForm.name} placeholder="AI Agent" required />
					</div>
					<Button type="submit" class="w-full" disabled={creating}>
						{creating ? 'Creating...' : 'Create Service Account'}
					</Button>
				</form>
			</Tabs.Content>
		</Tabs.Root>
	</Dialog.Content>
</Dialog.Root>

<!-- API key display dialog -->
<Dialog.Root open={!!showApiKey} onOpenChange={(open) => { if (!open) showApiKey = ''; }}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>API Key Created</Dialog.Title>
			<Dialog.Description>
				Copy this API key now. It will not be shown again.
			</Dialog.Description>
		</Dialog.Header>
		<div class="flex items-center gap-2">
			<Input value={showApiKey} readonly class="font-mono text-sm" />
			<Button variant="outline" size="sm" onclick={copyApiKey}>
				<Copy class="h-4 w-4" />
			</Button>
		</div>
	</Dialog.Content>
</Dialog.Root>
