import asyncio
import terminator

async def test():
    d = terminator.Desktop()
    locs = await d.locator('role:Window').all()
    print(f'Type of all(): {type(locs)}')
    print(f'Length: {len(locs)}')
    if locs:
        print(f'First item type: {type(locs[0])}')
        name = locs[0].name()
        print(f'Type of name(): {type(name)}')
        print(f'Name value: {name}')

asyncio.run(test())