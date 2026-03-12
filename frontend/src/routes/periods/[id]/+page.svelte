<script lang="ts">
	import { page } from '$app/stores';
	import { onMount } from 'svelte';
	import { periods, type FinancialPeriod, type ClosingResult, type JournalEntryDetail, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Badge } from '$lib/components/ui/badge';
	import { toast } from 'svelte-sonner';
	import { ArrowLeft, Lock, Eye } from '@lucide/svelte';

	let period = $state<FinancialPeriod | null>(null);
	let loading = $state(true);
	let previewEntry = $state<JournalEntryDetail | null>(null);
	let previewDialogOpen = $state(false);
	let closeDialogOpen = $state(false);
	let closing = $state(false);
	let previewing = $state(false);

	onMount(async () => {
		try {
			const res = await periods.get($page.params.id!);
			period = res.data;
		} catch {
			toast.error('Period not found');
		} finally {
			loading = false;
		}
	});

	async function handlePreview() {
		if (!period) return;
		previewing = true;
		try {
			const res = await periods.close(period.id, true);
			previewEntry = res.data.closing_entry ?? null;
			previewDialogOpen = true;
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to preview close');
		} finally {
			previewing = false;
		}
	}

	async function handleClose() {
		if (!period) return;
		closing = true;
		try {
			const res = await periods.close(period.id, false);
			period = res.data.period;
			closeDialogOpen = false;
			toast.success('Period closed');
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to close period');
		} finally {
			closing = false;
		}
	}
</script>

<svelte:head>
	<title>{period?.name ?? 'Period'} - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center gap-4">
		<a href="/periods" class="rounded-md p-1 hover:bg-muted">
			<ArrowLeft class="h-5 w-5" />
		</a>
		<h1 class="text-2xl font-semibold">{period?.name ?? 'Period'}</h1>
		{#if period}
			{#if period.closed_at}
				<Badge variant="secondary">Closed</Badge>
			{:else}
				<Badge variant="default">Open</Badge>
			{/if}
		{/if}
	</div>

	{#if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else if period}
		<div class="grid gap-6 lg:grid-cols-2">
			<Card.Root>
				<Card.Header class="pb-2">
					<Card.Title class="text-base">Details</Card.Title>
				</Card.Header>
				<Card.Content class="space-y-3">
					<div class="grid grid-cols-2 gap-4">
						<div>
							<p class="text-sm text-muted-foreground">Start Date</p>
							<p class="font-medium">{period.start_date}</p>
						</div>
						<div>
							<p class="text-sm text-muted-foreground">End Date</p>
							<p class="font-medium">{period.end_date}</p>
						</div>
					</div>
					{#if period.closed_at}
						<div>
							<p class="text-sm text-muted-foreground">Closed At</p>
							<p class="text-sm">{period.closed_at}</p>
						</div>
						{#if period.closing_entry_id}
							<div>
								<p class="text-sm text-muted-foreground">Closing Entry</p>
								<a href="/journal-entries/{period.closing_entry_id}" class="text-sm hover:underline">
									View closing entry
								</a>
							</div>
						{/if}
					{/if}
				</Card.Content>
			</Card.Root>

			{#if !period.closed_at}
				<Card.Root>
					<Card.Header class="pb-2">
						<Card.Title class="text-base">Actions</Card.Title>
					</Card.Header>
					<Card.Content class="space-y-3">
						<Button variant="outline" class="w-full" onclick={handlePreview} disabled={previewing}>
							<Eye class="mr-2 h-4 w-4" />
							{previewing ? 'Loading...' : 'Preview Close'}
						</Button>
						<Button variant="destructive" class="w-full" onclick={() => (closeDialogOpen = true)}>
							<Lock class="mr-2 h-4 w-4" />
							Close Period
						</Button>
						<p class="text-xs text-muted-foreground">
							Closing a period is permanent and cannot be undone. Revenue and expense
							accounts will be zeroed and net income transferred to retained earnings.
						</p>
					</Card.Content>
				</Card.Root>
			{/if}
		</div>
	{/if}
</div>

<!-- Preview dialog -->
<Dialog.Root bind:open={previewDialogOpen}>
	<Dialog.Content class="sm:max-w-2xl">
		<Dialog.Header>
			<Dialog.Title>Closing Entry Preview</Dialog.Title>
			<Dialog.Description>
				This entry would be created when closing the period.
			</Dialog.Description>
		</Dialog.Header>
		{#if previewEntry}
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Account</Table.Head>
						<Table.Head class="text-right">Debit</Table.Head>
						<Table.Head class="text-right">Credit</Table.Head>
					</Table.Row>
				</Table.Header>
				<Table.Body>
					{#each previewEntry.lines as line}
						<Table.Row>
							<Table.Cell>
								{line.account_name ?? line.account_id}
							</Table.Cell>
							<Table.Cell class="text-right font-mono">
								{line.display_debit && line.display_debit !== '0.00' ? line.display_debit : ''}
							</Table.Cell>
							<Table.Cell class="text-right font-mono">
								{line.display_credit && line.display_credit !== '0.00' ? line.display_credit : ''}
							</Table.Cell>
						</Table.Row>
					{/each}
				</Table.Body>
			</Table.Root>
		{:else}
			<p class="text-muted-foreground">No closing entry needed (no revenue/expense balances).</p>
		{/if}
	</Dialog.Content>
</Dialog.Root>

<!-- Close confirmation dialog -->
<Dialog.Root bind:open={closeDialogOpen}>
	<Dialog.Content class="sm:max-w-sm">
		<Dialog.Header>
			<Dialog.Title>Close Period?</Dialog.Title>
			<Dialog.Description>
				This action is permanent. The period cannot be reopened. All revenue and expense
				balances will be zeroed and net income transferred to retained earnings.
			</Dialog.Description>
		</Dialog.Header>
		<div class="flex justify-end gap-2">
			<Button variant="outline" onclick={() => (closeDialogOpen = false)}>Cancel</Button>
			<Button variant="destructive" onclick={handleClose} disabled={closing}>
				{closing ? 'Closing...' : 'Close Period'}
			</Button>
		</div>
	</Dialog.Content>
</Dialog.Root>
