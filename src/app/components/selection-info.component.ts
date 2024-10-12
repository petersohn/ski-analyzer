import { Component, ElementRef, Signal, computed, effect } from "@angular/core";
import { CommonModule } from "@angular/common";
import { MatCardModule } from "@angular/material/card";
import { MapService } from "@/services/map.service";
import { Lift, Piste } from "@/types/skiArea";
import { Activity } from "@/types/track";
import { NameValueComponent } from "./name-value.component";

const liftTypes: { [type: string]: string } = {
  cable_car: "Cable car",
  gondola: "Gondola",
  mixed_lift: "Mixed lfit",
  chair_lift: "Chairlift",
  drag_lift: "Draglift",
  "t-bar": "T-bar",
  "j-bar": "J-bar",
  platter: "Platter",
  rope_tow: "Rope tow",
  magic_carpet: "Magic carpet",
  zip_line: "Zipline",
};

const liftIcons: { [type: string]: string } = {
  cable_car: "cablecar",
  gondola: "gondola",
  mixed_lift: "gondola",
  chair_lift: "chairlift",
  drag_lift: "draglift",
  "t-bar": "draglift",
  "j-bar": "draglift",
  platter: "draglift",
  zip_line: "zipline",
};

const difficultyColors: { [type: string]: string } = {
  Novice: "#0a0",
  Easy: "#11f",
  Intermediate: "#f00",
  Advanced: "#000",
  Expert: "#000",
  Freeride: "#f60",
  Unknown: "#888",
};

const activityTypes: { [type: string]: string } = {
  Unknown: "unknown",
  UseLift: "Lift",
};

@Component({
  selector: "selection-info",
  standalone: true,
  imports: [CommonModule, MatCardModule, NameValueComponent],
  templateUrl: "./selection-info.component.html",
  styleUrls: ["./selection-info.component.css"],
})
export class SelectionInfoComponent {
  public selectedPiste: Signal<Piste | undefined>;
  public selectedLift: Signal<Lift | undefined>;
  public selectedActivity: Signal<Activity | undefined>;

  constructor(
    private readonly mapService: MapService,
    private readonly elementRef: ElementRef<HTMLElement>,
  ) {
    this.selectedPiste = this.mapService.selectedPiste;
    this.selectedLift = this.mapService.selectedLift;
    this.selectedActivity = this.mapService.selectedActivity;
    effect(() => {
      const color =
        difficultyColors[this.selectedPiste()?.difficulty ?? "Unknown"];
      this.elementRef.nativeElement.style.setProperty(
        "--difficulty-color",
        color,
      );
    });
  }

  public pisteName = computed(() => this.getName(this.selectedPiste()));

  public lift = computed(
    () => this.selectedLift() || this.selectedActivity()?.useLift?.lift,
  );
  public liftName = computed(() => this.getName(this.lift()));
  public liftType = computed(() => liftTypes[this.lift()?.type ?? ""]);
  public liftIcon = computed(() => {
    const type = liftIcons[this.lift()?.type ?? ""];
    if (type === undefined) {
      return;
    } else {
      return `/assets/lift/${type}.svg`;
    }
  });
  public stationCount = computed(
    () => "" + (this.lift()?.stations.length ?? 0),
  );

  public activityType = computed(
    () => activityTypes[this.selectedActivity()?.type ?? "Unknown"],
  );

  private getName(input?: { ref: string; name: string }) {
    if (!input) {
      return "";
    }

    if (input.ref && input.name) {
      return `[${input.ref}] ${input.name}`;
    } else {
      return input.name || input.ref || "";
    }
  }
}
