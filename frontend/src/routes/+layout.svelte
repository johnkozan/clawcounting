<script lang="ts">
	import '../app.css';
	import { page } from '$app/stores';
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';
	import { get } from 'svelte/store';
	import { authStore } from '$lib/stores/auth';
	import { Toaster } from '$lib/components/ui/sonner';
	import type { User } from '$lib/api';
	import { ModeWatcher } from 'mode-watcher';
	import { toggleMode, mode } from 'mode-watcher';
	import {
		LayoutDashboard,
		BookOpen,
		Landmark,
		FileText,
		CalendarRange,
		Coins,
		BarChart3,
		Settings,
		LogOut,
		Menu,
		X,
		Sun,
		Moon
	} from '@lucide/svelte';

	let { children } = $props();

	let sidebarOpen = $state(false);

	const navItems = [
		{ href: '/dashboard', label: 'Dashboard', icon: LayoutDashboard },
		{ href: '/accounts', label: 'Accounts', icon: Landmark },
		{ href: '/journal-entries', label: 'Journal Entries', icon: BookOpen },
		{ href: '/periods', label: 'Periods', icon: CalendarRange },
		{ href: '/currencies', label: 'Currencies', icon: Coins },
		{ href: '/reports', label: 'Reports', icon: BarChart3 },
		{ href: '/settings', label: 'Settings', icon: Settings }
	];

	let user = $state<User | null>(null);
	let loading = $state(true);

	$effect(() => {
		const unsub = authStore.user.subscribe((v) => { user = v; });
		return unsub;
	});

	$effect(() => {
		const unsub = authStore.loading.subscribe((v) => { loading = v; });
		return unsub;
	});

	const publicPaths = ['/login', '/register'];

	onMount(() => {
		authStore.initialize();

		const unsubLoading = authStore.loading.subscribe((isLoading) => {
			if (isLoading) return;
			const unsub = authStore.user.subscribe((u) => {
				const currentPath = window.location.pathname;
				if (!u && !publicPaths.some((p) => currentPath.startsWith(p))) {
					goto('/login');
				}
			});
			unsub();
		});

		return unsubLoading;
	});

	function handleLogout() {
		authStore.logout();
		goto('/login');
	}

	function isActive(href: string, currentPath: string): boolean {
		if (href === '/dashboard') return currentPath === '/dashboard' || currentPath === '/';
		return currentPath.startsWith(href);
	}
</script>

{#if loading}
	<div class="flex h-screen items-center justify-center">
		<div class="text-muted-foreground">Loading...</div>
	</div>
{:else if !user && !publicPaths.some((p) => $page.url.pathname.startsWith(p))}
	<div class="flex h-screen items-center justify-center">
		<div class="text-muted-foreground">Redirecting...</div>
	</div>
{:else if publicPaths.some((p) => $page.url.pathname.startsWith(p))}
	{@render children()}
{:else}
	<div class="flex h-screen overflow-hidden">
		<!-- Mobile sidebar backdrop -->
		{#if sidebarOpen}
			<button
				class="fixed inset-0 z-40 bg-black/50 lg:hidden"
				onclick={() => (sidebarOpen = false)}
				aria-label="Close sidebar"
			></button>
		{/if}

		<!-- Sidebar -->
		<aside
			class="fixed inset-y-0 left-0 z-50 flex w-64 flex-col border-r border-border bg-card transition-transform lg:static lg:translate-x-0 {sidebarOpen
				? 'translate-x-0'
				: '-translate-x-full'}"
		>
			<div class="flex h-14 items-center gap-2 border-b border-border px-4">
				<Landmark class="h-6 w-6 text-primary" />
				<span class="text-lg font-semibold">ClawCounting</span>
				<button class="ml-auto lg:hidden" onclick={() => (sidebarOpen = false)}>
					<X class="h-5 w-5" />
				</button>
			</div>

			<nav class="flex-1 overflow-y-auto p-3">
				<ul class="space-y-1">
					{#each navItems as item}
						{@const Icon = item.icon}
						<li>
							<a
								href={item.href}
								onclick={() => (sidebarOpen = false)}
								class="flex items-center gap-3 rounded-md px-3 py-2 text-sm font-medium transition-colors {isActive(
									item.href,
									$page.url.pathname
								)
									? 'bg-primary text-primary-foreground'
									: 'text-muted-foreground hover:bg-accent hover:text-accent-foreground'}"
							>
								<Icon class="h-4 w-4" />
								{item.label}
							</a>
						</li>
					{/each}
				</ul>
			</nav>

			<div class="border-t border-border p-3">
				<div class="flex items-center gap-3 px-3 py-2">
					<div class="flex h-8 w-8 items-center justify-center rounded-full bg-primary text-xs font-medium text-primary-foreground">
						{user?.name?.charAt(0)?.toUpperCase() ?? '?'}
					</div>
					<div class="flex-1 truncate">
						<div class="text-sm font-medium truncate">{user?.name ?? 'User'}</div>
						<div class="text-xs text-muted-foreground truncate">{user?.email ?? ''}</div>
					</div>
					<button
						onclick={toggleMode}
						class="rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground"
						title="Toggle theme"
					>
						{#if mode.current === 'light'}
							<Sun class="h-4 w-4" />
						{:else}
							<Moon class="h-4 w-4" />
						{/if}
					</button>
					<button
						onclick={handleLogout}
						class="rounded-md p-1.5 text-muted-foreground hover:bg-accent hover:text-accent-foreground"
						title="Sign out"
					>
						<LogOut class="h-4 w-4" />
					</button>
				</div>
			</div>
		</aside>

		<!-- Main content -->
		<div class="flex flex-1 flex-col overflow-hidden">
			<!-- Top bar (mobile) -->
			<header class="flex h-14 items-center gap-4 border-b border-border px-4 lg:hidden">
				<button onclick={() => (sidebarOpen = true)}>
					<Menu class="h-5 w-5" />
				</button>
				<span class="font-semibold">ClawCounting</span>
			</header>

			<main class="flex-1 overflow-y-auto p-6">
				{@render children()}
			</main>
		</div>
	</div>
{/if}

<ModeWatcher defaultMode="system" />
<Toaster />
