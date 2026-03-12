<script lang="ts">
	import { onMount } from 'svelte';
	import { journalEntries, periods, type JournalEntry, type FinancialPeriod } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { Input } from '$lib/components/ui/input';
	import { toast } from 'svelte-sonner';
	import { Plus } from '@lucide/svelte';

	let items = $state<JournalEntry[]>([]);
	let periodList = $state<FinancialPeriod[]>([]);
	let loading = $state(true);
	let hasMore = $state(false);
	let nextCursor = $state<string | undefined>();
	let periodFilter = $state<string>('');
	let startDate = $state('');
	let endDate = $state('');

	onMount(async () => {
		try {
			const [jeRes, pRes] = await Promise.all([
				journalEntries.list({ limit: 50 }),
				periods.list({ limit: 50 })
			]);
			items = jeRes.data;
			hasMore = jeRes.has_more;
			nextCursor = jeRes.next_cursor;
			periodList = pRes.data;
		} catch {
			toast.error('Failed to load journal entries');
		} finally {
			loading = false;
		}
	});

	async function applyFilters() {
		loading = true;
		try {
			const params: Record<string, string | number | undefined> = { limit: 50 };
			if (periodFilter) params.period_id = periodFilter;
			if (startDate) params.start_date = startDate;
			if (endDate) params.end_date = endDate;
			const res = await journalEntries.list(params);
			items = res.data;
			hasMore = res.has_more;
			nextCursor = res.next_cursor;
		} catch {
			toast.error('Failed to filter');
		} finally {
			loading = false;
		}
	}

	async function loadMore() {
		if (!hasMore || !nextCursor) return;
		try {
			const params: Record<string, string | number | undefined> = { limit: 50, cursor: nextCursor };
			if (periodFilter) params.period_id = periodFilter;
			const res = await journalEntries.list(params);
			items = [...items, ...res.data];
			hasMore = res.has_more;
			nextCursor = res.next_cursor;
		} catch {
			toast.error('Failed to load more');
		}
	}
</script>

<svelte:head>
	<title>Journal Entries - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-semibold">Journal Entries</h1>
		<a href="/journal-entries/new">
			<Button>
				<Plus class="mr-2 h-4 w-4" />
				New Entry
			</Button>
		</a>
	</div>

	<div class="flex flex-wrap gap-4">
		<Select.Root type="single" value={periodFilter} onValueChange={(v) => { periodFilter = v ?? ''; applyFilters(); }}>
			<Select.Trigger class="w-48">
				{periodList.find((p) => p.id === periodFilter)?.name ?? 'All periods'}
			</Select.Trigger>
			<Select.Content>
				<Select.Item value="">All periods</Select.Item>
				{#each periodList as p}
					<Select.Item value={p.id}>{p.name}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>
		<Input type="date" bind:value={startDate} class="w-40" placeholder="Start date" onchange={() => applyFilters()} />
		<Input type="date" bind:value={endDate} class="w-40" placeholder="End date" onchange={() => applyFilters()} />
	</div>

	<Card.Root>
		<Card.Content class="p-0">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Date</Table.Head>
						<Table.Head>Description</Table.Head>
						<Table.Head>Reference</Table.Head>
						<Table.Head>Type</Table.Head>
						<Table.Head>Created</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#if loading}
						<Table.Row>
							<Table.Cell colspan={5} class="text-center text-muted-foreground">
								Loading...
							</Table.Cell>
						</Table.Row>
					{:else if items.length === 0}
						<Table.Row>
							<Table.Cell colspan={5} class="text-center text-muted-foreground">
								No journal entries found.
							</Table.Cell>
						</Table.Row>
					{:else}
						{#each items as entry}
							<Table.Row class="cursor-pointer hover:bg-muted/50">
								<Table.Cell class="text-sm">
									<a href="/journal-entries/{entry.id}" class="hover:underline">
										{entry.entry_date}
									</a>
								</Table.Cell>
								<Table.Cell>
									<a href="/journal-entries/{entry.id}" class="font-medium hover:underline">
										{entry.description}
									</a>
								</Table.Cell>
								<Table.Cell class="text-sm text-muted-foreground">
									{entry.reference ?? '-'}
								</Table.Cell>
								<Table.Cell>
									{#if entry.is_reversal}
										<Badge variant="outline">Reversal</Badge>
									{/if}
								</Table.Cell>
								<Table.Cell class="text-sm text-muted-foreground">
									{entry.created_at.split('T')[0]}
								</Table.Cell>
							</Table.Row>
						{/each}
					{/if}
				</Table.Body>
			</Table.Root>
		</Card.Content>
	</Card.Root>

	{#if hasMore}
		<div class="text-center">
			<Button variant="outline" onclick={loadMore}>Load more</Button>
		</div>
	{/if}
</div>
