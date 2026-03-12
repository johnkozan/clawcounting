<script lang="ts">
	import { onMount } from 'svelte';
	import { periods, settings, type FinancialPeriod, type CreatePeriodRequest, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Table from '$lib/components/ui/table';
	import * as Dialog from '$lib/components/ui/dialog';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { Badge } from '$lib/components/ui/badge';
	import * as Alert from '$lib/components/ui/alert';
	import { toast } from 'svelte-sonner';
	import { Plus, AlertTriangle } from '@lucide/svelte';

	let items = $state<FinancialPeriod[]>([]);
	let loading = $state(true);
	let hasRetainedEarnings = $state(true);
	let dialogOpen = $state(false);
	let creating = $state(false);

	let form = $state<CreatePeriodRequest>({
		name: '',
		start_date: '',
		end_date: ''
	});

	const openPeriods = $derived(items.filter((p) => !p.closed_at));

	onMount(async () => {
		try {
			const [pRes, sRes] = await Promise.allSettled([
				periods.list({ limit: 50 }),
				settings.get()
			]);
			if (pRes.status === 'fulfilled') items = pRes.value.data;
			if (sRes.status === 'fulfilled') {
				hasRetainedEarnings = !!sRes.value.data.retained_earnings_account_id;
			}
		} catch {
			toast.error('Failed to load periods');
		} finally {
			loading = false;
		}
	});

	function setPreset(type: 'annual' | 'quarter') {
		const year = new Date().getFullYear();
		if (type === 'annual') {
			form.name = `FY${year}`;
			form.start_date = `${year}-01-01`;
			form.end_date = `${year}-12-31`;
		} else {
			const q = Math.ceil((new Date().getMonth() + 1) / 3);
			const startMonth = (q - 1) * 3 + 1;
			const endMonth = startMonth + 2;
			const lastDay = new Date(year, endMonth, 0).getDate();
			form.name = `Q${q} ${year}`;
			form.start_date = `${year}-${String(startMonth).padStart(2, '0')}-01`;
			form.end_date = `${year}-${String(endMonth).padStart(2, '0')}-${lastDay}`;
		}
	}

	async function handleCreate(e: Event) {
		e.preventDefault();
		creating = true;
		try {
			const res = await periods.create(form);
			items = [res.data, ...items];
			toast.success('Period created');
			dialogOpen = false;
			form = { name: '', start_date: '', end_date: '' };
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to create period');
		} finally {
			creating = false;
		}
	}
</script>

<svelte:head>
	<title>Financial Periods - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<div class="flex items-center justify-between">
		<h1 class="text-2xl font-semibold">Financial Periods</h1>
		<Button onclick={() => (dialogOpen = true)}>
			<Plus class="mr-2 h-4 w-4" />
			New Period
		</Button>
	</div>

	{#if !hasRetainedEarnings}
		<Alert.Root variant="destructive">
			<AlertTriangle class="h-4 w-4" />
			<Alert.Title>Retained Earnings Not Set</Alert.Title>
			<Alert.Description>
				You must configure a retained earnings account in
				<a href="/settings" class="underline">Settings</a> before closing periods.
			</Alert.Description>
		</Alert.Root>
	{/if}

	{#if !loading && openPeriods.length === 0 && items.length > 0}
		<Alert.Root>
			<AlertTriangle class="h-4 w-4" />
			<Alert.Title>No Open Periods</Alert.Title>
			<Alert.Description>
				All periods are closed. Create a new period to continue recording journal entries.
			</Alert.Description>
		</Alert.Root>
	{/if}

	<Card.Root>
		<Card.Content class="p-0">
			<Table.Root>
				<Table.Header>
					<Table.Row>
						<Table.Head>Name</Table.Head>
						<Table.Head>Start Date</Table.Head>
						<Table.Head>End Date</Table.Head>
						<Table.Head>Status</Table.Head>
						<Table.Head>Closed At</Table.Head>
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
								No periods yet. Create a financial period to start.
							</Table.Cell>
						</Table.Row>
					{:else}
						{#each items as period}
							<Table.Row class="cursor-pointer hover:bg-muted/50">
								<Table.Cell>
									<a href="/periods/{period.id}" class="font-medium hover:underline">
										{period.name}
									</a>
								</Table.Cell>
								<Table.Cell>{period.start_date}</Table.Cell>
								<Table.Cell>{period.end_date}</Table.Cell>
								<Table.Cell>
									{#if period.closed_at}
										<Badge variant="secondary">Closed</Badge>
									{:else}
										<Badge variant="default">Open</Badge>
									{/if}
								</Table.Cell>
								<Table.Cell class="text-sm text-muted-foreground">
									{period.closed_at ?? '-'}
								</Table.Cell>
							</Table.Row>
						{/each}
					{/if}
				</Table.Body>
			</Table.Root>
		</Card.Content>
	</Card.Root>
</div>

<Dialog.Root bind:open={dialogOpen}>
	<Dialog.Content class="sm:max-w-md">
		<Dialog.Header>
			<Dialog.Title>New Financial Period</Dialog.Title>
			<Dialog.Description>Create a new financial period for recording entries.</Dialog.Description>
		</Dialog.Header>
		<form onsubmit={handleCreate} class="space-y-4">
			<div class="flex gap-2">
				<Button type="button" variant="outline" size="sm" onclick={() => setPreset('annual')}>
					Annual
				</Button>
				<Button type="button" variant="outline" size="sm" onclick={() => setPreset('quarter')}>
					Current Quarter
				</Button>
			</div>
			<div class="space-y-2">
				<Label for="periodName">Name</Label>
				<Input id="periodName" bind:value={form.name} placeholder="FY2026" required />
			</div>
			<div class="grid grid-cols-2 gap-4">
				<div class="space-y-2">
					<Label for="startDate">Start Date</Label>
					<Input id="startDate" type="date" bind:value={form.start_date} required />
				</div>
				<div class="space-y-2">
					<Label for="endDate">End Date</Label>
					<Input id="endDate" type="date" bind:value={form.end_date} required />
				</div>
			</div>
			<div class="flex justify-end gap-2">
				<Button variant="outline" type="button" onclick={() => (dialogOpen = false)}>Cancel</Button>
				<Button type="submit" disabled={creating}>
					{creating ? 'Creating...' : 'Create Period'}
				</Button>
			</div>
		</form>
	</Dialog.Content>
</Dialog.Root>
