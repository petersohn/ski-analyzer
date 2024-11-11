import { Component, ElementRef, Signal, computed, effect } from "@angular/core";
import { CommonModule } from "@angular/common";
import { MatCardModule } from "@angular/material/card";
import { MapService } from "@/services/map.service";
import { Lift, Piste } from "@/types/skiArea";
import { Activity, Waypoint } from "@/types/track";
import { NameValueComponent } from "./name-value.component";
import { Dayjs } from "dayjs";

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
  public selectedWaypoint: Signal<Waypoint | undefined>;
  public currentWaypointSpeed: Signal<number | undefined>;
  public currentWaypointClosestLift: Signal<
    { lift: Lift; distance: number } | undefined
  >;

  constructor(
    private readonly mapService: MapService,
    private readonly elementRef: ElementRef<HTMLElement>,
  ) {
    this.selectedPiste = this.mapService.selectedPiste;
    this.selectedLift = this.mapService.selectedLift;
    this.selectedActivity = this.mapService.selectedActivity;
    this.selectedWaypoint = this.mapService.selectedWaypoint;
    this.currentWaypointSpeed = this.mapService.currentWaypointSpeed;
    this.currentWaypointClosestLift =
      this.mapService.currentWaypointClosestLift;
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
  public liftLength = computed(() =>
    this.lift()?.lengths.length === 1
      ? this.meters(this.lift()?.lengths[0] ?? 0)
      : "",
  );
  public liftLengths = computed((): string[] => {
    const lift = this.lift();
    if ((lift?.lengths.length ?? 0) > 1) {
      return lift!.lengths.map((l) => this.meters(l));
    } else {
      return [];
    }
  });

  public activityType = computed(
    () => activityTypes[this.selectedActivity()?.type ?? "Unknown"],
  );
  public activityLength = computed(() =>
    this.meters(this.selectedActivity()?.length ?? 0),
  );
  public activityTime = computed(
    () =>
      this.getTime(this.selectedActivity()?.begin_time) +
      " - " +
      this.getTime(this.selectedActivity()?.end_time),
  );

  public waypointTime = computed(() =>
    this.getTime(this.selectedWaypoint()?.time),
  );

  public waypointAccuracy = computed(() => {
    const hdop = this.selectedWaypoint()?.hdop;
    return hdop !== undefined ? this.meters(hdop) : "";
  });

  public waypointSpeed = computed(() => {
    const speed = this.selectedWaypoint()?.speed;
    return !!speed ? this.metersPerSecond(speed) : "";
  });

  public speed = computed(() => {
    const speed = this.currentWaypointSpeed();
    return !!speed ? this.metersPerSecond(speed) : "";
  });

  public closestLift = computed(() => {
    const lift = this.currentWaypointClosestLift();
    return lift !== undefined
      ? `${lift.lift.name} (${this.meters(lift.distance)})`
      : "";
  });

  private meters(len: number) {
    return Math.round(len) + " m";
  }

  private metersPerSecond(speed: number) {
    return speed.toFixed(1) + " m/s";
  }

  private getTime(time?: Dayjs | null): string {
    return time?.format("HH:mm:ss") ?? "???";
  }

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
