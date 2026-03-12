<script lang="ts">
	import { onMount } from 'svelte';
	import { reports, periods, type BalanceSheetReport, type FinancialPeriod } from '$lib/api';
	import { downloadCsv } from '$lib/csv';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft, Download } from '@lucide/svelte';

	let report = $state<BalanceSheetReport | null>(null);
	let periodList = $state<FinancialPeriod[]>([]);
	let loading = $state(true);
	let selectedPeriod = $state<string>('');
	let asOfDate = $state('');

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
			if (asOfDate) params.as_of_date = asOfDate;
			const res = await reports.balanceSheet(params);
			report = res.data;
		} catch {
			toast.error('Failed to load balance sheet');
		} finally {
			loading = false;
		}
	}

	function exportCsv() {
		if (!report) return;
		const rows: string[][] = [];
		for (const section of report.sections) {
			rows.push([section.name, '', '']);
			for (const row of section.rows) {
				rows.push([`  ${row.account_number} ${row.account_name}`, '', row.display_balance]);
			}
			rows.push([`Total ${section.name}`, '', section.display_total]);
			rows.push(['', '', '']);
		}
		downloadCsv('balance-sheet.csv', ['Account', '', 'Balance'], rows);
	}
</script>

<svelte:head>
	<title>Balance Sheet - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/reports" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">Balance Sheet</h1>
	</div>

	<div class="flex items-center gap-4">
		<Select.Root type="single" value={selectedPeriod} onValueChange={(v) => { selectedPeriod = v ?? ''; asOfDate = ''; loadReport(); }}>
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
		<div class="flex items-center gap-2">
			<Label for="asOf" class="text-sm whitespace-nowrap">As of:</Label>
			<Input id="asOf" type="date" bind:value={asOfDate} class="w-40" onchange={() => { selectedPeriod = ''; loadReport(); }} />
		</div>
		<Button variant="outline" size="sm" onclick={exportCsv} disabled={!report}>
			<Download class="mr-2 h-4 w-4" />
			CSV
		</Button>
	</div>

	{#if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else if report}
		<div class="grid gap-6">
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
										<Table.Cell class="text-right font-mono">{row.display_balance}</Table.Cell>
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
				<Card.Content class="p-4">
					<div class="grid grid-cols-3 gap-4 text-center">
						<div>
							<p class="text-sm text-muted-foreground">Assets</p>
							<p class="text-xl font-semibold">{report.display_total_assets}</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">=</p>
							<p class="text-xl">=</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">Liabilities + Equity</p>
							<p class="text-xl font-semibold">{report.display_total_liabilities} + {report.display_total_equity}</p>
						</div>
					</div>
				</Card.Content>
			</Card.Root>
		</div>
	{/if}
</div>
