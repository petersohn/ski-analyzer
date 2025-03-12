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
  private cachedSkiAreaMap: Map<string, CachedSkiArea> = new Map();
  private loadId = 0;

  public actionOnSelect: (() => void | Promise<void>) | null = null;

  public clearChoosableSkiAreas(): void {
    ++this.loadId;
    this.actionOnSelect = null;
    this.loadedSkiAreas.set([]);
    this.cachedSkiAreaMap = new Map();
    this.updateCachedSkiAreas();
  }

  public removeCachedSkiArea(uuid: string): void {
    this.cachedSkiAreaMap.delete(uuid);
    this.updateCachedSkiAreas();
  }

  public async selectSkiAreas(
    cachedP: Promise<CachedSkiArea[]>,
    loadedP: Promise<SkiAreaMetadata[] | undefined>,
  ) {
    const loadId = ++this.loadId;
    this.loadedSkiAreas.set(null);

    const cached = await cachedP;
    if (loadId !== this.loadId) {
      return;
    }

    this.cachedSkiAreaMap = new Map(cached.map((c) => [c.uuid, c]));
    this.updateCachedSkiAreas();

    const loaded = await loadedP;
    if (loadId !== this.loadId) {
      return;
    }

    this.loadedSkiAreas.set(loaded);
  }

  private updateCachedSkiAreas() {
    this.cachedSkiAreas.set(Array.from(this.cachedSkiAreaMap.values()));
  }
}
