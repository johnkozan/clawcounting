<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { accounts, type Account, type AccountBalance, type JournalEntryLine, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft } from '@lucide/svelte';

	let account = $state<Account | null>(null);
	let balance = $state<AccountBalance | null>(null);
	let transactions = $state<JournalEntryLine[]>([]);
	let subAccounts = $state<Account[]>([]);
	let loading = $state(true);
	let saving = $state(false);
	let editName = $state('');

	onMount(async () => {
		const id = $page.params.id!;
		try {
			const [accRes, balRes, txRes] = await Promise.allSettled([
				accounts.get(id),
				accounts.balance(id),
				accounts.transactions(id, { limit: 50 })
			]);

			if (accRes.status === 'fulfilled') {
				account = accRes.value.data;
				editName = account.name;
				if (accRes.value.data.has_subledger) {
					const subRes = await accounts.subAccounts(id);
					subAccounts = subRes.data;
				}
			}
			if (balRes.status === 'fulfilled') balance = balRes.value.data;
			if (txRes.status === 'fulfilled') transactions = txRes.value.data;
		} catch {
			toast.error('Failed to load account');
		} finally {
			loading = false;
		}
	});

	async function handleSave(e: Event) {
		e.preventDefault();
		if (!account) return;
		saving = true;
		try {
			const res = await accounts.update(account.id, { name: editName });
			account = res.data;
			toast.success('Account updated');
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to update');
		} finally {
			saving = false;
		}
	}

	async function toggleActive() {
		if (!account) return;
		try {
			const res = await accounts.update(account.id, { is_active: !account.is_active });
			account = res.data;
			toast.success(account.is_active ? 'Account activated' : 'Account deactivated');
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
		}
	}
</script>

<svelte:head>
	<title>{account?.name ?? 'Account'} - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/accounts" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">
			{#if account}
				<span class="font-mono text-muted-foreground">{account.account_number}</span>
				{account.name}
			{:else}
				Account
			{/if}
		</h1>
		{#if account}
			<Badge variant={account.is_active ? 'secondary' : 'destructive'}>
				{account.is_active ? 'Active' : 'Inactive'}
			</Badge>
			{#if account.has_subledger}
				<Badge variant="outline">Control Account</Badge>
			{/if}
		{/if}
	</div>

	{#if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else if account}
		<div class="grid gap-6 lg:grid-cols-3">
			<!-- Balance card -->
			<Card.Root>
				<Card.Header class="pb-2">
					<Card.Title class="text-base">Balance</Card.Title>
				</Card.Header>
				<Card.Content>
					{#if balance}
						<div class="text-3xl font-semibold">{balance.display_balance}</div>
						<div class="mt-2 space-y-1 text-sm text-muted-foreground">
							<div>Debits: {balance.display_debits}</div>
							<div>Credits: {balance.display_credits}</div>
						</div>
					{:else}
						<div class="text-muted-foreground">No balance data</div>
					{/if}
				</Card.Content>
			</Card.Root>

			<!-- Edit card -->
			<Card.Root class="lg:col-span-2">
				<Card.Header class="pb-2">
					<Card.Title class="text-base">Details</Card.Title>
				</Card.Header>
				<Card.Content>
					<form onsubmit={handleSave} class="space-y-4">
						<div class="grid grid-cols-2 gap-4">
							<div class="space-y-2">
								<Label for="name">Name</Label>
								<Input id="name" bind:value={editName} required />
							</div>
							<div class="space-y-2">
								<Label>Type</Label>
								<Input value={account.account_type} disabled />
							</div>
						</div>
						<div class="flex gap-2">
							<Button type="submit" size="sm" disabled={saving}>
								{saving ? 'Saving...' : 'Save'}
							</Button>
							<Button type="button" variant="outline" size="sm" onclick={toggleActive}>
								{account.is_active ? 'Deactivate' : 'Activate'}
							</Button>
							{#if account.has_subledger}
								<a href="/accounts/{account.id}/subledger">
									<Button type="button" variant="outline" size="sm">Manage Subledger</Button>
								</a>
							{/if}
						</div>
					</form>
				</Card.Content>
			</Card.Root>
		</div>

		<!-- Sub-accounts (if control account) -->
		{#if account.has_subledger && subAccounts.length > 0}
			<Card.Root>
				<Card.Header>
					<Card.Title class="text-base">Sub-Accounts</Card.Title>
				</Card.Header>
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
						</Table.Body>
					</Table.Root>
				</Card.Content>
			</Card.Root>
		{/if}

		<!-- Transaction history -->
		<Card.Root>
			<Card.Header>
				<Card.Title class="text-base">Transaction History</Card.Title>
			</Card.Header>
			<Card.Content class="p-0">
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>Date</Table.Head>
							<Table.Head>Description</Table.Head>
							<Table.Head class="text-right">Debit</Table.Head>
							<Table.Head class="text-right">Credit</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#if transactions.length === 0}
							<Table.Row>
								<Table.Cell colspan={4} class="text-center text-muted-foreground">
									No transactions
								</Table.Cell>
							</Table.Row>
						{:else}
							{#each transactions as tx}
								<Table.Row>
									<Table.Cell class="text-sm">
										<a href="/journal-entries/{tx.journal_entry_id}" class="hover:underline">
											{tx.description ?? '-'}
										</a>
									</Table.Cell>
									<Table.Cell>{tx.description ?? '-'}</Table.Cell>
									<Table.Cell class="text-right font-mono">
										{tx.display_debit && tx.display_debit !== '0' ? tx.display_debit : ''}
									</Table.Cell>
									<Table.Cell class="text-right font-mono">
										{tx.display_credit && tx.display_credit !== '0' ? tx.display_credit : ''}
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
