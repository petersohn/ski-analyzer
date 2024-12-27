import { Injectable, signal, computed } from "@angular/core";
import { SkiAreaMetadata } from "@/types/skiArea";
import { CachedSkiArea } from "@/types/config";

@Injectable({ providedIn: "root" })
export class SkiAreaChooserService {
  public loadedSkiAreas = signal<SkiAreaMetadata[] | null | undefined>([]);
  public cachedSkiAreas = signal<CachedSkiArea[]>([]);
  public hasChoosableSkiArea = computed(
    () =>
      (this.loadedSkiAreas() !== null && this.loadedSkiAreas()?.length !== 0) ||
      this.cachedSkiAreas().length !== 0,
  );

  public clearChoosableSkiAreas(): void {
    this.loadedSkiAreas.set([]);
    this.cachedSkiAreas.set([]);
  }

  public async selectSkiAreas(
    cachedP: Promise<CachedSkiArea[]>,
    loadedP: Promise<SkiAreaMetadata[]>,
  ) {
    this.loadedSkiAreas.set(null);
    this.cachedSkiAreas.set(await cachedP);
    this.loadedSkiAreas.set(await loadedP);
  }
}
