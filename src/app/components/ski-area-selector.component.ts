import {
  Component,
  Signal,
  computed,
  signal,
  HostListener,
  ViewChild,
  ElementRef,
  AfterViewInit,
} from "@angular/core";
import { MatListModule } from "@angular/material/list";
import { MatInputModule } from "@angular/material/input";
import { MatFormFieldModule } from "@angular/material/form-field";
import { FormsModule } from "@angular/forms";
import { MatButtonModule } from "@angular/material/button";
import { SkiAreaMetadata } from "@/types/skiArea";
import { ActionsService } from "@/services/actions.service";
import { SkiAreaChooserService } from "@/services/ski-area-chooser.service";
import { MapService } from "@/services/map.service";
import { filterString } from "@/utils/string";
import { CachedSkiArea } from "@/types/config";

@Component({
  selector: "ski-area-selector",
  templateUrl: "./ski-area-selector.component.html",
  styleUrl: "./ski-area-selector.component.css",
  standalone: true,
  imports: [
    MatListModule,
    MatButtonModule,
    FormsModule,
    MatFormFieldModule,
    MatInputModule,
  ],
})
export class SkiAreaSelectorComponent implements AfterViewInit {
  public loadedSkiAreas: Signal<SkiAreaMetadata[] | null | undefined>;
  public cachedSkiAreas: Signal<CachedSkiArea[]>;
  public filter = signal("");
  public displayedLoadedSkiAreas = computed(() =>
    this.loadedSkiAreas()?.filter((a) => filterString(a.name, this.filter())),
  );
  public displayedCachedSkiAreas = computed(() =>
    this.cachedSkiAreas().filter((a) =>
      filterString(a.metadata.name, this.filter()),
    ),
  );
  public canHaveLoadedSkiAreas = computed(
    () => this.loadedSkiAreas() === undefined,
  );
  public isLoading = computed(() => this.loadedSkiAreas() === null);

  @ViewChild("search")
  private searchInput!: ElementRef<HTMLInputElement>;

  private focusedListItem: SkiAreaMetadata | undefined;
  private hoveredListItem: SkiAreaMetadata | undefined;

  constructor(
    private readonly actionsService: ActionsService,
    private readonly mapService: MapService,
    private readonly skiAreaChooserService: SkiAreaChooserService,
  ) {
    this.loadedSkiAreas = this.skiAreaChooserService.loadedSkiAreas;
    this.cachedSkiAreas = this.skiAreaChooserService.cachedSkiAreas;
  }

  public ngAfterViewInit() {
    this.searchInput.nativeElement.focus();
  }

  @HostListener("window:keyup.escape")
  public onEscape() {
    this.cancel();
  }

  @HostListener("window:keyup.enter")
  public onEnter() {
    const loaded = this.displayedLoadedSkiAreas();
    const cached = this.displayedCachedSkiAreas();
    const loadedCount = loaded?.length ?? 0;
    const cachedCount = cached.length;

    if (
      cachedCount === 1 &&
      (loadedCount === 0 ||
        (loadedCount === 1 && loaded![0].id === cached[0].metadata.id))
    ) {
      this.acceptCached(cached[0].uuid);
    } else if (loadedCount === 1 && cachedCount === 0) {
      this.acceptLoaded(loaded![0].id);
    }
  }

  public acceptLoaded(id: number) {
    this.actionsService.loadSkiAreaFromId(id);
    this.close();
  }

  public acceptCached(uuid: string) {
    console.log("acceptCached", uuid);
    this.actionsService.loadCachedSkiArea(uuid);
    this.close();
  }

  public cancel() {
    this.close();
  }

  public close() {
    this.mapService.clearOutline();
    this.skiAreaChooserService.clearChoosableSkiAreas();
  }

  public focusListItem(skiArea: SkiAreaMetadata) {
    this.focusedListItem = skiArea;
    this.highlight(skiArea);
  }

  public blurListItem() {
    this.focusedListItem = undefined;
    this.highlight(this.hoveredListItem);
  }

  public hoverListItem(skiArea: SkiAreaMetadata) {
    this.hoveredListItem = skiArea;
    this.highlight(skiArea);
  }

  public unhoverListItem() {
    this.hoveredListItem = undefined;
    this.highlight(this.focusedListItem);
  }

  private highlight(skiArea: SkiAreaMetadata | undefined) {
    if (!!skiArea) {
      this.mapService.addOutline(skiArea.outline.item);
    } else {
      this.mapService.clearOutline();
    }
  }
}
