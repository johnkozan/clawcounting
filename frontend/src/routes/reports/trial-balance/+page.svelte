<script lang="ts">
	import { onMount } from 'svelte';
	import { reports, periods, type TrialBalanceReport, type FinancialPeriod } from '$lib/api';
	import { downloadCsv } from '$lib/csv';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft, Download } from '@lucide/svelte';

	let report = $state<TrialBalanceReport | null>(null);
	let periodList = $state<FinancialPeriod[]>([]);
	let loading = $state(true);
	let selectedPeriod = $state<string>('');

	onMount(async () => {
		try {
			const pRes = await periods.list({ limit: 50 });
			periodList = pRes.data;
			await loadReport();
		} catch {
			toast.error('Failed to load');
		}
	});

	async function loadReport() {
		loading = true;
		try {
			const params: Record<string, string> = {};
			if (selectedPeriod) params.period_id = selectedPeriod;
			const res = await reports.trialBalance(params);
			report = res.data;
		} catch {
			toast.error('Failed to load trial balance');
		} finally {
			loading = false;
		}
	}

	function exportCsv() {
		if (!report) return;
		downloadCsv(
			'trial-balance.csv',
			['Account Number', 'Account Name', 'Type', 'Debit', 'Credit'],
			report.rows.map((r) => [r.account_number, r.account_name, r.account_type, r.display_debit, r.display_credit])
		);
	}
</script>

<svelte:head>
	<title>Trial Balance - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/reports" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">Trial Balance</h1>
	</div>

	<div class="flex items-center gap-4">
		<Select.Root type="single" value={selectedPeriod} onValueChange={(v) => { selectedPeriod = v ?? ''; loadReport(); }}>
			<Select.Trigger class="w-48">
				{periodList.find((p) => p.id === selectedPeriod)?.name ?? 'All periods'}
			</Select.Trigger>
			<Select.Content>
				<Select.Item value="">All periods</Select.Item>
				{#each periodList as p}
					<Select.Item value={p.id}>{p.name}</Select.Item>
				{/each}
			</Select.Content>
		</Select.Root>
		<Button variant="outline" size="sm" onclick={exportCsv} disabled={!report}>
			<Download class="mr-2 h-4 w-4" />
			CSV
		</Button>
	</div>

	<Card.Root>
		<Card.Content class="p-0">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Account Number</Table.Head>
						<Table.Head>Account Name</Table.Head>
						<Table.Head>Type</Table.Head>
						<Table.Head class="text-right">Debit</Table.Head>
						<Table.Head class="text-right">Credit</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#if loading}
						<Table.Row>
							<Table.Cell colspan={5} class="text-center text-muted-foreground">Loading...</Table.Cell>
						</Table.Row>
					{:else if !report || report.rows.length === 0}
						<Table.Row>
							<Table.Cell colspan={5} class="text-center text-muted-foreground">No data</Table.Cell>
						</Table.Row>
					{:else}
						{#each report.rows as row}
							<Table.Row>
								<Table.Cell class="font-mono">{row.account_number}</Table.Cell>
								<Table.Cell>{row.account_name}</Table.Cell>
								<Table.Cell class="text-sm text-muted-foreground">{row.account_type}</Table.Cell>
								<Table.Cell class="text-right font-mono">{row.display_debit}</Table.Cell>
								<Table.Cell class="text-right font-mono">{row.display_credit}</Table.Cell>
							</Table.Row>
						{/each}
						<Table.Row class="bg-muted/50 font-semibold">
							<Table.Cell colspan={3}>Totals</Table.Cell>
							<Table.Cell class="text-right font-mono">{report.display_total_debits}</Table.Cell>
							<Table.Cell class="text-right font-mono">{report.display_total_credits}</Table.Cell>
						</Table.Row>
					{/if}
				</Table.Body>
			</Table.Root>
		</Card.Content>
	</Card.Root>
</div>
