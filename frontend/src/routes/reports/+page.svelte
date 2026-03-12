<script lang="ts">
	import * as Card from '$lib/components/ui/card';
	import { BarChart3, Scale, TrendingUp, BookOpen } from '@lucide/svelte';

	const reportLinks = [
		{
			href: '/reports/trial-balance',
			title: 'Trial Balance',
			description: 'Verify that total debits equal total credits across all accounts.',
			icon: Scale
		},
		{
			href: '/reports/balance-sheet',
			title: 'Balance Sheet',
			description: 'Assets, liabilities, and equity at a point in time.',
			icon: BarChart3
		},
		{
			href: '/reports/income-statement',
			title: 'Income Statement',
			description: 'Revenue, expenses, and net income for a period.',
			icon: TrendingUp
		},
		{
			href: '/reports/general-ledger',
			title: 'General Ledger',
			description: 'Detailed transaction history with running balances.',
			icon: BookOpen
		}
	];
</script>

<svelte:head>
	<title>Reports - ClawCounting</title>
</svelte:head>

<div class="space-y-6">
	<h1 class="text-2xl font-semibold">Reports</h1>

	<div class="grid gap-4 sm:grid-cols-2">
		{#each reportLinks as report}
			{@const Icon = report.icon}
			<a href={report.href}>
				<Card.Root class="transition-colors hover:bg-muted/50">
					<Card.Content class="flex items-start gap-4 p-6">
						<div class="flex h-10 w-10 items-center justify-center rounded-lg bg-primary/10">
							<Icon class="h-5 w-5 text-primary" />
						</div>
						<div>
							<h2 class="font-semibold">{report.title}</h2>
							<p class="mt-1 text-sm text-muted-foreground">{report.description}</p>
						</div>
					</Card.Content>
				</Card.Root>
			</a>
		{/each}
	</div>
</div>
