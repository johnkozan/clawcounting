<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { goto } from '$app/navigation';
	import { accounts, currencies, type Account, type Currency, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft, Plus } from '@lucide/svelte';

	let parent = $state<Account | null>(null);
	let subAccounts = $state<Account[]>([]);
	let loading = $state(true);
	let dialogOpen = $state(false);
	let creating = $state(false);

	let newName = $state('');
	let newNumber = $state('');
	let newEntityId = $state('');

	onMount(async () => {
		const id = $page.params.id!;
		try {
			const [parentRes, subsRes] = await Promise.all([
				accounts.get(id),
				accounts.subAccounts(id)
			]);
			parent = parentRes.data;
			subAccounts = subsRes.data;
		} catch {
			toast.error('Failed to load account');
		} finally {
			loading = false;
		}
	});

	async function createSubAccount(e: Event) {
		e.preventDefault();
		if (!parent) return;
		creating = true;
		try {
			await accounts.create({
				currency_id: parent.currency_id,
				account_number: newNumber,
				name: newName,
				account_type: parent.account_type,
				normal_balance: parent.normal_balance,
				parent_id: parent.id,
				entity_id: newEntityId
			});
			toast.success('Sub-account created');
			dialogOpen = false;
			newName = '';
			newNumber = '';
			newEntityId = '';
			const subsRes = await accounts.subAccounts(parent.id);
			subAccounts = subsRes.data;
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to create sub-account');
		} finally {
			creating = false;
		}
	}
</script>

<svelte:head>
	<title>Subledger - {parent?.name ?? 'Account'} - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/accounts/{$page.params.id}" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">
			Subledger: {parent?.name ?? ''}
		</h1>
	</div>

	<div class="flex justify-end">
		<Button onclick={() => (dialogOpen = true)}>
			<Plus class="mr-2 h-4 w-4" />
			New Sub-Account
		</Button>
	</div>

	{#if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else}
		<Card.Root>
			<Card.Content class="p-0">
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>Number</Table.Head>
							<Table.Head>Name</Table.Head>
							<Table.Head>Entity ID</Table.Head>
							<Table.Head>Status</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#if subAccounts.length === 0}
							<Table.Row>
								<Table.Cell colspan={4} class="text-center text-muted-foreground">
									No sub-accounts yet.
								</Table.Cell>
							</Table.Row>
						{:else}
							{#each subAccounts as sub}
								<Table.Row>
									<Table.Cell class="font-mono">
										<a href="/accounts/{sub.id}" class="hover:underline">{sub.account_number}</a>
									</Table.Cell>
									<Table.Cell>{sub.name}</Table.Cell>
									<Table.Cell class="text-sm text-muted-foreground">{sub.entity_id ?? '-'}</Table.Cell>
									<Table.Cell>
										<Badge variant={sub.is_active ? 'secondary' : 'destructive'}>
											{sub.is_active ? 'Active' : 'Inactive'}
										</Badge>
									</Table.Cell>
								</Table.Row>
							{/each}
						{/if}
					</Table.Body>
				</Table.Root>
			</Card.Content>
		</Card.Root>
	{/if}
</div>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>New Sub-Account</Dialog.Title>
			<Dialog.Description>Create a sub-account under {parent?.name ?? 'this account'}.</Dialog.Description>
		</Dialog.Header>
		<form onsubmit={createSubAccount} class="space-y-4">
			<div class="space-y-2">
				<Label for="subNumber">Account Number</Label>
				<Input id="subNumber" bind:value={newNumber} placeholder="1000-001" required />
			</div>
			<div class="space-y-2">
				<Label for="subName">Name</Label>
				<Input id="subName" bind:value={newName} placeholder="Customer: Acme Corp" required />
			</div>
			<div class="space-y-2">
				<Label for="entityId">Entity ID</Label>
				<Input id="entityId" bind:value={newEntityId} placeholder="acme-corp" required />
			</div>
			<div class="flex justify-end gap-2">
				<Button variant="outline" type="button" onclick={() => (dialogOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={creating}>
					{creating ? 'Creating...' : 'Create'}
				</Button>
			</div>
		</form>
	</Dialog.Content>
</Dialog.Root>
