export function downloadCsv(filename: string, headers: string[], rows: string[][]) {
	const escape = (val: string) => {
		if (val.includes(',') || val.includes('"') || val.includes('\n')) {
			return `"${val.replace(/"/g, '""')}"`;
		}
		return val;
	};

	const csv = [headers.map(escape).join(','), ...rows.map((r) => r.map(escape).join(','))].join(
		'\n'
	);

	const blob = new Blob([csv], { type: 'text/csv;charset=utf-8;' });
	const url = URL.createObjectURL(blob);
	const a = document.createElement('a');
	a.href = url;
	a.download = filename;
	a.click();
	URL.revokeObjectURL(url);
}
