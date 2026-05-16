export function getStatusColor(status: string) {
  if (status === 'ok' || status === 'ready') {
    return 'green';
  }

  if (status === 'degraded' || status === 'not_configured' || status === 'pending') {
    return 'yellow';
  }

  if (status === 'disabled') {
    return 'gray';
  }

  return 'red';
}
