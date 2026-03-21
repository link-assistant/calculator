import { describe, it, expect } from 'vitest';
import { createRateCoordination } from './worker-rate-coordination';

describe('createRateCoordination', () => {
  it('allRatesReady resolves only after all three sources resolve', async () => {
    const coord = createRateCoordination();
    let resolved = false;

    const promise = coord.allRatesReady().then(() => { resolved = true; });

    // Not resolved yet — no sources done
    await Promise.resolve(); // flush microtasks
    expect(resolved).toBe(false);

    coord.resolveEcb();
    await Promise.resolve();
    expect(resolved).toBe(false);

    coord.resolveCbr();
    await Promise.resolve();
    expect(resolved).toBe(false);

    coord.resolveCrypto();
    await promise;
    expect(resolved).toBe(true);
  });

  it('allRatesReady resolves instantly when all sources are already loaded', async () => {
    const coord = createRateCoordination();

    coord.resolveEcb();
    coord.resolveCbr();
    coord.resolveCrypto();

    // Should resolve immediately (no waiting)
    await coord.allRatesReady();
  });

  it('resolving sources in any order works', async () => {
    const coord = createRateCoordination();

    coord.resolveCrypto();
    coord.resolveEcb();
    coord.resolveCbr();

    await coord.allRatesReady();
  });

  it('reset creates new pending promises', async () => {
    const coord = createRateCoordination();

    // Resolve all, confirm ready
    coord.resolveEcb();
    coord.resolveCbr();
    coord.resolveCrypto();
    await coord.allRatesReady();

    // Reset — should require resolving again
    coord.reset();

    let resolved = false;
    const promise = coord.allRatesReady().then(() => { resolved = true; });

    await Promise.resolve();
    expect(resolved).toBe(false);

    coord.resolveEcb();
    coord.resolveCbr();
    coord.resolveCrypto();
    await promise;
    expect(resolved).toBe(true);
  });

  it('calling resolve multiple times is harmless', async () => {
    const coord = createRateCoordination();

    coord.resolveEcb();
    coord.resolveEcb(); // double call
    coord.resolveCbr();
    coord.resolveCrypto();

    await coord.allRatesReady();
  });

  it('multiple allRatesReady callers all resolve', async () => {
    const coord = createRateCoordination();

    const p1 = coord.allRatesReady();
    const p2 = coord.allRatesReady();

    coord.resolveEcb();
    coord.resolveCbr();
    coord.resolveCrypto();

    await Promise.all([p1, p2]);
  });
});
