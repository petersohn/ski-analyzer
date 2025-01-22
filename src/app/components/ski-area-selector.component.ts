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
import { MatButtonModule } from "@angular/material/button";
import { MatIconModule } from "@angular/material/icon";
import { MatDialog } from "@angular/material/dialog";
import { FormsModule } from "@angular/forms";
import { SkiAreaMetadata } from "@/types/skiArea";
import { ActionsService } from "@/services/actions.service";
import { SkiAreaChooserService } from "@/services/ski-area-chooser.service";
import { MapService } from "@/services/map.service";
import { filterString } from "@/utils/string";
import { CachedSkiArea } from "@/types/config";
import { CachedSkiAreaItem } from "./cached-ski-area-item.component";
import {
  ConfirmationDialogData,
  ConfirmationDialogComponent,
  ConfirmationDialogOption,
} from "./confirmation-dialog.component";
import { lastValueFrom } from "rxjs";

@Component({
  selector: "ski-area-selector",
  templateUrl: "./ski-area-selector.component.html",
  styleUrl: "./ski-area-selector.component.scss",
  imports: [
    MatListModule,
    MatButtonModule,
    FormsModule,
    MatFormFieldModule,
    MatInputModule,
    CachedSkiAreaItem,
    MatIconModule,
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
    private readonly dialog: MatDialog,
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
      this.acceptLoaded(loaded![0]);
    }
  }

  public async acceptLoaded(skiArea: SkiAreaMetadata) {
    const cached = this.cachedSkiAreas().filter(
      (s) => s.metadata.id === skiArea.id,
    );
    if (cached.length !== 0) {
      const isMultipleCached = cached.length > 1;
      let replaceOptions: ConfirmationDialogOption[];

      if (isMultipleCached) {
        cached.sort((a, b) => b.date.diff(a.date));
        replaceOptions = [
          {
            text: "Relpace all cached",
            value: "replace_all",
          },
          {
            text: "Relpace latest cached",
            value: "replace_latest",
          },
        ];
      } else {
        replaceOptions = [
          {
            text: "Relpace cached",
            value: "replace_latest",
          },
        ];
      }

      const dialogRef = this.dialog.open<
        ConfirmationDialogComponent,
        ConfirmationDialogData
      >(ConfirmationDialogComponent, {
        data: {
          text: `The ski area ${skiArea.name} was already cached at ${cached[0].date.format("YYYY-MM-DD")}. Loading it again will create a new cache and keep the old one.`,
          options: [
            {
              text: isMultipleCached ? "Load latest cached" : "Load cached",
              value: "cached",
              default: true,
            },
            {
              text: "Load new",
              value: "load",
            },
            ...replaceOptions,
            {
              text: "Cancel",
            },
          ],
        },
      });
      const result = await lastValueFrom(dialogRef.afterClosed());
      switch (result) {
        case "cached":
          this.acceptCached(cached[0].uuid);
          return;
        case "load":
          break;
        case "replace_all":
          for (const skiArea of cached) {
            this.deleteCached(skiArea);
          }
          break;
        case "replace_latest":
          this.deleteCached(cached[0]);
          break;
        default:
          return;
      }
    }
    this.actionsService.loadSkiAreaFromId(skiArea.id);
    this.close();
  }

  public acceptCached(uuid: string) {
    this.actionsService.loadCachedSkiArea(uuid);
    this.close();
  }

  public deleteCached(skiArea: CachedSkiArea) {
    this.actionsService.removeCachedSkiArea(skiArea.uuid);
    this.skiAreaChooserService.removeCachedSkiArea(skiArea.uuid);
    if (this.hoveredListItem?.id === skiArea.metadata.id) {
      this.unhoverListItem();
    }
    if (this.focusedListItem?.id === skiArea.metadata.id) {
      this.blurListItem();
    }
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
