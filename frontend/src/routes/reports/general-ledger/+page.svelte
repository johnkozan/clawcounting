<script lang="ts">
	import { onMount } from 'svelte';
	import { reports, accounts as accountsApi, type GeneralLedgerReport, type Account } from '$lib/api';
	import { downloadCsv } from '$lib/csv';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft, Download } from '@lucide/svelte';

	let report = $state<GeneralLedgerReport | null>(null);
	let accountList = $state<Account[]>([]);
	let loading = $state(false);
	let selectedAccount = $state<string>('');
	let startDate = $state('');
	let endDate = $state('');

	onMount(async () => {
		try {
			const res = await accountsApi.list({ limit: 200 });
			accountList = res.data;
		} catch {
			toast.error('Failed to load accounts');
		}
	});

	async function loadReport() {
		if (!selectedAccount) return;
		loading = true;
		try {
			const res = await reports.generalLedger({
				account_id: selectedAccount,
				limit: 200,
				start_date: startDate || undefined,
				end_date: endDate || undefined
			});
			report = res.data;
		} catch {
			toast.error('Failed to load general ledger');
		} finally {
			loading = false;
		}
	}

	function exportCsv() {
		if (!report) return;
		downloadCsv(
			'general-ledger.csv',
			['Date', 'Description', 'Reference', 'Debit', 'Credit', 'Running Balance'],
			report.entries.map((e) => [
				e.entry_date,
				e.description,
				e.reference ?? '',
				e.display_debit,
				e.display_credit,
				e.display_running_balance
			])
		);
	}

	function getAccountLabel(id: string): string {
		const acc = accountList.find((a) => a.id === id);
		return acc ? `${acc.account_number} - ${acc.name}` : 'Select account';
	}
</script>

<svelte:head>
	<title>General Ledger - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/reports" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">General Ledger</h1>
	</div>

	<div class="flex flex-wrap items-end gap-4">
		<div class="w-64">
			<Label class="text-sm">Account</Label>
			<Select.Root type="single" value={selectedAccount} onValueChange={(v) => { if (v) { selectedAccount = v; loadReport(); } }}>
				<Select.Trigger class="w-full">
					{getAccountLabel(selectedAccount)}
				</Select.Trigger>
				<Select.Content>
					{#each accountList as acc}
						<Select.Item value={acc.id}>{acc.account_number} - {acc.name}</Select.Item>
					{/each}
				</Select.Content>
			</Select.Root>
		</div>
		<div>
			<Label for="glStart" class="text-sm">Start Date</Label>
			<Input id="glStart" type="date" bind:value={startDate} class="w-40" onchange={loadReport} />
		</div>
		<div>
			<Label for="glEnd" class="text-sm">End Date</Label>
			<Input id="glEnd" type="date" bind:value={endDate} class="w-40" onchange={loadReport} />
		</div>
		<Button variant="outline" size="sm" onclick={exportCsv} disabled={!report}>
			<Download class="mr-2 h-4 w-4" />
			CSV
		</Button>
	</div>

	{#if !selectedAccount}
		<Card.Root>
			<Card.Content class="py-12 text-center text-muted-foreground">
				Select an account to view its general ledger.
			</Card.Content>
		</Card.Root>
	{:else if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else if report}
		<Card.Root>
			<Card.Header class="py-3">
				<div class="flex items-center justify-between">
					<Card.Title class="text-base">{report.account_name}</Card.Title>
					<span class="text-sm text-muted-foreground">
						Starting balance: {report.display_starting_balance}
					</span>
				</div>
			</Card.Header>
			<Card.Content class="p-0">
				<Table.Root>
					<Table.Header>
						<Table.Row>
							<Table.Head>Date</Table.Head>
							<Table.Head>Description</Table.Head>
							<Table.Head>Reference</Table.Head>
							<Table.Head class="text-right">Debit</Table.Head>
							<Table.Head class="text-right">Credit</Table.Head>
							<Table.Head class="text-right">Balance</Table.Head>
						</Table.Row>
					</Table.Header>
					<Table.Body>
						{#if report.entries.length === 0}
							<Table.Row>
								<Table.Cell colspan={6} class="text-center text-muted-foreground">
									No transactions in this range.
								</Table.Cell>
							</Table.Row>
						{:else}
							{#each report.entries as entry}
								<Table.Row>
									<Table.Cell class="text-sm">
										<a href="/journal-entries/{entry.entry_id}" class="hover:underline">
											{entry.entry_date}
										</a>
									</Table.Cell>
									<Table.Cell>{entry.description}</Table.Cell>
									<Table.Cell class="text-sm text-muted-foreground">{entry.reference ?? ''}</Table.Cell>
									<Table.Cell class="text-right font-mono">
										{entry.display_debit !== '0.00' ? entry.display_debit : ''}
									</Table.Cell>
									<Table.Cell class="text-right font-mono">
										{entry.display_credit !== '0.00' ? entry.display_credit : ''}
									</Table.Cell>
									<Table.Cell class="text-right font-mono font-medium">
										{entry.display_running_balance}
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
