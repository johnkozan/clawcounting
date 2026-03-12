<script lang="ts">
	import { onMount } from 'svelte';
	import { accounts, type Account, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { toast } from 'svelte-sonner';
	import { Plus, Search } from '@lucide/svelte';

	let items = $state<Account[]>([]);
	let loading = $state(true);
	let search = $state('');
	let typeFilter = $state<string>('');

	const accountTypes = ['asset', 'liability', 'equity', 'revenue', 'expense'];

	const filtered = $derived(
		items.filter((a) => {
			const matchesSearch =
				!search ||
				a.name.toLowerCase().includes(search.toLowerCase()) ||
				a.account_number.toLowerCase().includes(search.toLowerCase());
			const matchesType = !typeFilter || a.account_type === typeFilter;
			return matchesSearch && matchesType;
		})
	);

	const grouped = $derived(() => {
		const groups: Record<string, Account[]> = {};
		for (const a of filtered) {
			if (!groups[a.account_type]) groups[a.account_type] = [];
			groups[a.account_type].push(a);
		}
		return groups;
	});

	onMount(async () => {
		try {
			const res = await accounts.list({ limit: 200 });
			items = res.data;
		} catch {
			toast.error('Failed to load accounts');
		} finally {
			loading = false;
		}
	});

	const typeColors: Record<string, string> = {
		asset: 'bg-emerald-100 text-emerald-800',
		liability: 'bg-red-100 text-red-800',
		equity: 'bg-blue-100 text-blue-800',
		revenue: 'bg-purple-100 text-purple-800',
		expense: 'bg-amber-100 text-amber-800'
	};
</script>

<svelte:head>
	<title>Chart of Accounts - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-semibold">Chart of Accounts</h1>
		<a href="/accounts/new">
			<Button>
				<Plus class="mr-2 h-4 w-4" />
				New Account
			</Button>
		</a>
	</div>

	<div class="flex gap-4">
		<div class="relative flex-1">
			<Search class="absolute left-3 top-3 h-4 w-4 text-muted-foreground" />
			<Input placeholder="Search accounts..." bind:value={search} class="pl-9" />
		</div>
		<div class="flex gap-1">
			<Button
				variant={typeFilter === '' ? 'default' : 'outline'}
				size="sm"
				onclick={() => (typeFilter = '')}
			>
				All
			</Button>
			{#each accountTypes as t}
				<Button
					variant={typeFilter === t ? 'default' : 'outline'}
					size="sm"
					onclick={() => (typeFilter = t)}
				>
					{t.charAt(0).toUpperCase() + t.slice(1)}
				</Button>
			{/each}
		</div>
	</div>

	{#if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else}
		{#each accountTypes as accountType}
			{@const groupAccounts = filtered.filter((a) => a.account_type === accountType)}
			{#if groupAccounts.length > 0}
				<Card.Root>
					<Card.Header class="py-3">
						<Card.Title class="flex items-center gap-2 text-base">
							<span
								class="inline-flex rounded px-2 py-0.5 text-xs font-medium {typeColors[accountType]}"
							>
								{accountType.charAt(0).toUpperCase() + accountType.slice(1)}
							</span>
							<span class="text-sm text-muted-foreground">({groupAccounts.length})</span>
						</Card.Title>
					</Card.Header>
					<Card.Content class="p-0">
						<Table.Root>
							<Table.Header>
								<Table.Row>
									<Table.Head>Number</Table.Head>
									<Table.Head>Name</Table.Head>
									<Table.Head>Normal</Table.Head>
									<Table.Head>Subledger</Table.Head>
									<Table.Head>Status</Table.Head>
								</Table.Row>
							</Table.Header>
							<Table.Body>
								{#each groupAccounts as account}
									<Table.Row class="cursor-pointer hover:bg-muted/50">
										<Table.Cell class="font-mono">
											<a href="/accounts/{account.id}" class="hover:underline">
												{account.account_number}
											</a>
										</Table.Cell>
										<Table.Cell>
											<a href="/accounts/{account.id}" class="font-medium hover:underline">
												{account.name}
											</a>
											{#if account.parent_id}
												<span class="ml-1 text-xs text-muted-foreground">(sub-account)</span>
											{/if}
										</Table.Cell>
										<Table.Cell class="text-sm text-muted-foreground">
											{account.normal_balance}
										</Table.Cell>
										<Table.Cell>
											{#if account.has_subledger}
												<Badge variant="outline">Control</Badge>
											{/if}
										</Table.Cell>
										<Table.Cell>
											{#if account.is_active}
												<Badge variant="secondary">Active</Badge>
											{:else}
												<Badge variant="destructive">Inactive</Badge>
											{/if}
										</Table.Cell>
									</Table.Row>
								{/each}
							</Table.Body>
						</Table.Root>
					</Card.Content>
				</Card.Root>
			{/if}
		{/each}

		{#if filtered.length === 0}
			<Card.Root>
				<Card.Content class="py-12 text-center text-muted-foreground">
					{items.length === 0 ? 'No accounts yet. Create your first account to get started.' : 'No accounts match your search.'}
				</Card.Content>
			</Card.Root>
		{/if}
	{/if}
</div>
