<script lang="ts">
	import { onMount } from 'svelte';
	import { settings, accounts as accountsApi, type Settings, type Account, ApiError } from '$lib/api';
	import * as Card from '$lib/components/ui/card';
	import * as Select from '$lib/components/ui/select';
	import { Button } from '$lib/components/ui/button';
	import { Input } from '$lib/components/ui/input';
	import { Label } from '$lib/components/ui/label';
	import { toast } from 'svelte-sonner';

	let currentSettings = $state<Settings | null>(null);
	let accountList = $state<Account[]>([]);
	let loading = $state(true);
	let saving = $state(false);
	let instanceName = $state('');
	let retainedEarningsId = $state('');

	onMount(async () => {
		try {
			const [sRes, aRes] = await Promise.all([
				settings.get(),
				accountsApi.list({ limit: 200, account_type: 'equity' })
			]);
			currentSettings = sRes.data;
			accountList = aRes.data;
			instanceName = currentSettings.instance_name ?? '';
			retainedEarningsId = currentSettings.retained_earnings_account_id ?? '';
		} catch {
			toast.error('Failed to load settings');
		} finally {
			loading = false;
		}
	});

	async function handleSave(e: Event) {
		e.preventDefault();
		saving = true;
		try {
			const res = await settings.update({
				instance_name: instanceName,
				retained_earnings_account_id: retainedEarningsId || undefined
			});
			currentSettings = res.data;
			toast.success('Settings saved');
		} catch (err) {
			if (err instanceof ApiError) toast.error(err.message);
			else toast.error('Failed to save');
		} finally {
			saving = false;
		}
	}
</script>

<svelte:head>
	<title>Settings - ClawCounting</title>
</svelte:head>

<div class="mx-auto max-w-2xl space-y-6">
	<h1 class="text-2xl font-semibold">Settings</h1>

	{#if loading}
		<div class="text-muted-foreground">Loading...</div>
	{:else}
		<Card.Root>
			<Card.Header>
				<Card.Title class="text-base">Instance Settings</Card.Title>
			</Card.Header>
			<Card.Content>
				<form onsubmit={handleSave} class="space-y-4">
					<div class="space-y-2">
						<Label for="instanceName">Instance Name</Label>
						<Input id="instanceName" bind:value={instanceName} placeholder="ClawCounting" />
					</div>

					<div class="space-y-2">
						<Label>Retained Earnings Account</Label>
						<p class="text-xs text-muted-foreground">
							Required for period close. Must be an equity account.
						</p>
						<Select.Root type="single" value={retainedEarningsId} onValueChange={(v) => { retainedEarningsId = v ?? ''; }}>
							<Select.Trigger class="w-full">
								{accountList.find((a) => a.id === retainedEarningsId)
									? `${accountList.find((a) => a.id === retainedEarningsId)?.account_number} - ${accountList.find((a) => a.id === retainedEarningsId)?.name}`
									: 'Select equity account'}
							</Select.Trigger>
							<Select.Content>
								{#each accountList as acc}
									<Select.Item value={acc.id}>{acc.account_number} - {acc.name}</Select.Item>
								{/each}
							</Select.Content>
						</Select.Root>
					</div>

					<Button type="submit" disabled={saving}>
						{saving ? 'Saving...' : 'Save Settings'}
					</Button>
				</form>
			</Card.Content>
		</Card.Root>

		<Card.Root>
			<Card.Header>
				<Card.Title class="text-base">User Management</Card.Title>
			</Card.Header>
			<Card.Content>
				<a href="/settings/users">
					<Button variant="outline">Manage Users</Button>
				</a>
			</Card.Content>
		</Card.Root>
	{/if}
</div>
