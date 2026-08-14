export function Results({ items, state }: Props) @{
    <main>
        @if (state === "loading") {
            <Loading />
        } @else {
            @for (const item of items; index index; key item.id) {
                <Row item={item} index={index} />
            } @empty {
                <Empty />
            }
        }
        @switch (state) {
            @case "ready": { <Ready /> }
            @default: { <Idle /> }
        }
        @try {
            <Content />
        } @pending {
            <Loading />
        } @catch (error, reset) {
            <ErrorView error={error} reset={reset} />
        }
    </main>
}
