import dayjs from "dayjs";
import { Dayjs } from "dayjs";
import { Point, Rect } from "./geo";
import { Lift, SkiArea } from "./skiArea";

export type RawWaypoint = {
  point: Point;
  time: string;
};

export type RawUseLift = {
  lift: string;
  begin_time: string | null;
  end_time: string | null;
  begin_station: number;
  end_station: number;
  is_reverse: boolean;
};

export type RawActivityType = {
  Unknown?: null;
  UseLift?: RawUseLift;
};

export type RawActivity = {
  type: RawActivityType;
  route: RawWaypoint[][];
};

export type RawTrack = {
  item: RawActivity[];
  bounding_rect: Rect;
};

export type Waypoint = {
  point: Point;
  time: Dayjs;
};

export type Segment = Waypoint[];
export type Segments = Segment[];

export type UseLift = {
  lift: Lift;
  begin_time: Dayjs | null;
  end_time: Dayjs | null;
  begin_station: number;
  end_station: number;
  is_reverse: boolean;
};

export type ActivityType = "Unknown" | "UseLift";

export type Activity = {
  type: ActivityType;
  useLift?: UseLift;
  route: Segments;
};

export type Track = {
  item: Activity[];
  bounding_rect: Rect;
};

export class TrackConverter {
  constructor(private readonly skiArea: SkiArea) {}

  public convertTrack(route: RawTrack): Track {
    return {
      item: route.item.map((activity) => {
        return {
          type: this.convertActivityType(activity.type),
          useLift: this.convertUseLift(activity.type.UseLift),
          route: this.convertRoute(activity.route),
        };
      }),
      bounding_rect: route.bounding_rect,
    };
  }

  private convertUseLift(input?: RawUseLift): UseLift | undefined {
    if (!input) {
      return;
    }

    const lift = this.skiArea.lifts.get(input.lift);
    if (!lift) {
      throw new Error(`Lift not found with id: ${input.lift}`);
    }

    return {
      lift,
      begin_time: dayjs(input.begin_time),
      end_time: dayjs(input.end_time),
      begin_station: input.begin_station,
      end_station: input.end_station,
      is_reverse: input.is_reverse,
    };
  }

  private convertRoute(route: RawWaypoint[][]): Segments {
    return route.map((s) =>
      s.map((wp) => {
        return {
          point: wp.point,
          time: dayjs(wp.time),
        };
      }),
    );
  }

  private convertActivityType(type: RawActivityType): ActivityType {
    return (Object.keys(type)[0] ?? "Unknown") as ActivityType;
  }
}
