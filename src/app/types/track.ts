import dayjs from "dayjs";
import { Dayjs } from "dayjs";
import { Point, Rect } from "./geo";
import { Lift, SkiArea } from "./skiArea";

export type RawWaypoint = {
  point: Point;
  time?: string;
  hdop?: number;
};

export type RawUseLift = {
  lift: string;
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
  begin_time: string | null;
  end_time: string | null;
  length: number;
};

export type RawTrack = {
  item: RawActivity[];
  bounding_rect: Rect;
};

export type Waypoint = {
  point: Point;
  time?: Dayjs;
  hdop?: number;
  speed?: number;
};

export type Segment = Waypoint[];
export type Segments = Segment[];

export type UseLift = {
  lift: Lift;
  begin_station: number;
  end_station: number;
  is_reverse: boolean;
};

export type ActivityType = "Unknown" | "UseLift";

export type Activity = {
  type: ActivityType;
  useLift?: UseLift;
  route: Segments;
  begin_time: Dayjs | null;
  end_time: Dayjs | null;
  length: number;
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
          begin_time: dayjs(activity.begin_time),
          end_time: dayjs(activity.end_time),
          length: activity.length,
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
          time: wp.time !== undefined ? dayjs(wp.time) : undefined,
          hdop: wp.hdop,
        };
      }),
    );
  }

  private convertActivityType(type: RawActivityType): ActivityType {
    return (Object.keys(type)[0] ?? "Unknown") as ActivityType;
  }
}
