<script lang="ts">
	import { onMount } from 'svelte';
	import { reports, journalEntries, periods, type BalanceSheetReport, type JournalEntry, type FinancialPeriod } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import { Badge } from '$lib/components/ui/badge';
	import { TrendingUp, TrendingDown, DollarSign, Scale, FileText, CalendarRange } from '@lucide/svelte';

	let balanceSheet = $state<BalanceSheetReport | null>(null);
	let recentEntries = $state<JournalEntry[]>([]);
	let openPeriods = $state<FinancialPeriod[]>([]);
	let loading = $state(true);
	let error = $state('');

	onMount(async () => {
		try {
			const [bsRes, jeRes, pRes] = await Promise.allSettled([
				reports.balanceSheet({}),
				journalEntries.list({ limit: 10 }),
				periods.list({ limit: 50 })
			]);

			if (bsRes.status === 'fulfilled') balanceSheet = bsRes.value.data;
			if (jeRes.status === 'fulfilled') recentEntries = jeRes.value.data;
			if (pRes.status === 'fulfilled') openPeriods = pRes.value.data.filter((p) => !p.closed_at);
		} catch (err) {
			error = 'Failed to load dashboard data';
		} finally {
			loading = false;
		}
	});

	const stats = $derived([
		{
			label: 'Total Assets',
			value: balanceSheet?.display_total_assets ?? '-',
			icon: DollarSign,
			color: 'text-emerald-600'
		},
		{
			label: 'Total Liabilities',
			value: balanceSheet?.display_total_liabilities ?? '-',
			icon: TrendingDown,
			color: 'text-red-500'
		},
		{
			label: 'Total Equity',
			value: balanceSheet?.display_total_equity ?? '-',
			icon: Scale,
			color: 'text-blue-600'
		},
		{
			label: 'Open Periods',
			value: String(openPeriods.length),
			icon: CalendarRange,
			color: 'text-amber-600'
		}
	]);
</script>

<svelte:head>
	<title>Dashboard - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<h1 class="text-2xl font-semibold">Dashboard</h1>

	{#if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else}
		<!-- Stats cards -->
		<div class="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
			{#each stats as stat}
				{@const Icon = stat.icon}
				<Card.Root>
					<Card.Content class="flex items-center gap-4 p-6">
						<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-muted">
							<Icon class="h-5 w-5 {stat.color}" />
						</div>
						<div>
							<p class="text-sm text-muted-foreground">{stat.label}</p>
							<p class="text-xl font-semibold">{stat.value}</p>
						</div>
					</Card.Content>
				</Card.Root>
			{/each}
		</div>

		<div class="grid gap-6 lg:grid-cols-2">
			<!-- Recent journal entries -->
			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2 text-base">
						<FileText class="h-4 w-4" />
						Recent Journal Entries
					</Card.Title>
				</Card.Header>
				<Card.Content>
					{#if recentEntries.length === 0}
						<p class="text-sm text-muted-foreground">No journal entries yet.</p>
					{:else}
						<div class="space-y-3">
							{#each recentEntries as entry}
								<a
									href="/journal-entries/{entry.id}"
									class="flex items-center justify-between rounded-md p-2 hover:bg-muted"
								>
									<div>
										<p class="text-sm font-medium">{entry.description}</p>
										<p class="text-xs text-muted-foreground">{entry.entry_date}</p>
									</div>
									{#if entry.is_reversal}
										<Badge variant="outline">Reversal</Badge>
									{/if}
								</a>
							{/each}
						</div>
					{/if}
				</Card.Content>
			</Card.Root>

			<!-- Open periods -->
			<Card.Root>
				<Card.Header>
					<Card.Title class="flex items-center gap-2 text-base">
						<CalendarRange class="h-4 w-4" />
						Open Periods
					</Card.Title>
				</Card.Header>
				<Card.Content>
					{#if openPeriods.length === 0}
						<div class="rounded-md bg-amber-50 p-3 text-sm text-amber-800">
							No open periods. Create a financial period to start recording entries.
						</div>
					{:else}
						<div class="space-y-3">
							{#each openPeriods as period}
								<a
									href="/periods/{period.id}"
									class="flex items-center justify-between rounded-md p-2 hover:bg-muted"
								>
									<div>
										<p class="text-sm font-medium">{period.name}</p>
										<p class="text-xs text-muted-foreground">
											{period.start_date} to {period.end_date}
										</p>
									</div>
									<Badge variant="secondary">Open</Badge>
								</a>
							{/each}
						</div>
					{/if}
				</Card.Content>
			</Card.Root>
		</div>
	{/if}
</div>
