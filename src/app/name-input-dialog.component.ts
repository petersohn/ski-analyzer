import { Component, Inject, ViewChild, ElementRef } from "@angular/core";
import { MAT_DIALOG_DATA, MatDialogRef } from "@angular/material/dialog";
import { MatInput } from "@angular/material/input";
import { MatFormFieldModule } from "@angular/material/form-field";
import { MatButtonModule } from "@angular/material/button";

export type NameInputDialogData = {
  label: string;
  placeholder: string;
};

@Component({
  selector: "name-input-dialog",
  templateUrl: "./name-input-dialog.component.html",
  styleUrl: "./name-input-dialog.component.css",
  standalone: true,
  imports: [MatInput, MatFormFieldModule, MatButtonModule],
})
export class NameInputDialogComponent {
  @ViewChild("name")
  public name!: ElementRef<HTMLInputElement>;

  constructor(
    @Inject(MAT_DIALOG_DATA) public data: NameInputDialogData,
    private readonly dialogRef: MatDialogRef<NameInputDialogComponent>,
  ) {}

  public accept() {
    this.dialogRef.close(this.name.nativeElement.value);
  }

  public cancel() {
    this.dialogRef.close(null);
  }
}
