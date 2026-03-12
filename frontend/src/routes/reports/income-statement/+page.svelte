<script lang="ts">
	import { onMount } from 'svelte';
	import { reports, periods, type IncomeStatementReport, type FinancialPeriod } from '$lib/api';
	import { downloadCsv } from '$lib/csv';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft, Download } from '@lucide/svelte';

	let report = $state<IncomeStatementReport | null>(null);
	let periodList = $state<FinancialPeriod[]>([]);
	let loading = $state(true);
	let selectedPeriod = $state<string>('');

	onMount(async () => {
		try {
			const pRes = await periods.list({ limit: 50 });
			periodList = pRes.data;
			if (periodList.length > 0) {
				selectedPeriod = periodList[0].id;
				await loadReport();
			} else {
				loading = false;
			}
		} catch {
			toast.error('Failed to load');
			loading = false;
		}
	});

	async function loadReport() {
		if (!selectedPeriod) return;
		loading = true;
		try {
			const res = await reports.incomeStatement({ period_id: selectedPeriod });
			report = res.data;
		} catch {
			toast.error('Failed to load income statement');
		} finally {
			loading = false;
		}
	}

	function exportCsv() {
		if (!report) return;
		const rows: string[][] = [];
		for (const section of report.sections) {
			rows.push([section.name, '']);
			for (const row of section.rows) {
				rows.push([`  ${row.account_number} ${row.account_name}`, row.display_amount]);
			}
			rows.push([`Total ${section.name}`, section.display_total]);
			rows.push(['', '']);
		}
		rows.push(['Net Income', report.display_net_income]);
		downloadCsv('income-statement.csv', ['Account', 'Amount'], rows);
	}
</script>

<svelte:head>
	<title>Income Statement - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/reports" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">Income Statement</h1>
	</div>

	<div class="flex items-center gap-4">
		<Select.Root type="single" value={selectedPeriod} onValueChange={(v) => { if (v) { selectedPeriod = v; loadReport(); } }}>
			<Select.Trigger class="w-48">
				{periodList.find((p) => p.id === selectedPeriod)?.name ?? 'Select period'}
			</Select.Trigger>
			<Select.Content>
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

	{#if !selectedPeriod}
		<Card.Root>
			<Card.Content class="py-12 text-center text-muted-foreground">
				Select a period to view the income statement.
			</Card.Content>
		</Card.Root>
	{:else if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else if report}
		{#each report.sections as section}
			<Card.Root>
				<Card.Header class="py-3">
					<Card.Title class="text-base">{section.name}</Card.Title>
				</Card.Header>
				<Card.Content class="p-0">
					<Table.Root>
						<Table.Body>
							{#each section.rows as row}
								<Table.Row>
									<Table.Cell>
										<span class="font-mono text-muted-foreground">{row.account_number}</span>
										{row.account_name}
									</Table.Cell>
									<Table.Cell class="text-right font-mono">{row.display_amount}</Table.Cell>
								</Table.Row>
							{/each}
							<Table.Row class="bg-muted/50 font-semibold">
								<Table.Cell>Total {section.name}</Table.Cell>
								<Table.Cell class="text-right font-mono">{section.display_total}</Table.Cell>
							</Table.Row>
						</Table.Body>
					</Table.Root>
				</Card.Content>
			</Card.Root>
		{/each}

		<Card.Root>
			<Card.Content class="flex items-center justify-between p-6">
				<span class="text-lg font-semibold">Net Income</span>
				<span class="text-2xl font-bold">{report.display_net_income}</span>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
