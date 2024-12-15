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
import { MapService } from "@/services/map.service";
import { filterString } from "@/utils/string";

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
  public skiAreas: Signal<SkiAreaMetadata[]>;
  public filter = signal("");
  public displayedSkiAreas = computed(() =>
    this.skiAreas().filter((a) => filterString(a.name, this.filter())),
  );

  @ViewChild("search")
  private searchInput!: ElementRef<HTMLInputElement>;

  private focusedListItem: SkiAreaMetadata | undefined;
  private hoveredListItem: SkiAreaMetadata | undefined;

  constructor(
    private readonly actionsService: ActionsService,
    private readonly mapService: MapService,
  ) {
    this.skiAreas = this.actionsService.choosableSkiAreas;
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
    let skiAreas = this.displayedSkiAreas();
    if (skiAreas.length == 1) {
      this.accept(skiAreas[0].id);
    }
  }

  public accept(id: number) {
    this.actionsService.loadSkiAreaFromId(id);
    this.close();
  }

  public cancel() {
    this.close();
  }

  public close() {
    this.mapService.clearOutline();
    this.actionsService.choosableSkiAreas.set([]);
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
      this.mapService.addOutline(skiArea.outline);
    } else {
      this.mapService.clearOutline();
    }
  }
}
