export function Dynamic({ as: Tag, value }: Props) @{
    <{Tag} {value} data-value={value}></{Tag}>
}
