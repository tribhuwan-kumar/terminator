import terminator
import inspect
import asyncio

async def test():
    d = terminator.Desktop()
    loc = d.locator('role:Window')

    # Check all()
    try:
        all_result = await loc.all()
        print(f'Type of all() result: {type(all_result)}')
        print(f'Length: {len(all_result)}')
    except Exception as e:
        print(f'all() threw error: {e}')
        all_result = []

    # Check first()
    first_result = await loc.first()
    print(f'\nType of first() result: {type(first_result)}')

    if first_result:
        # Check element methods
        print(f'\nElement methods:')
        print(f'  name() returns: {type(first_result.name())}')
        print(f'  is_enabled() returns: {type(first_result.is_enabled())}')
        print(f'  is_focused() returns: {type(first_result.is_focused())}')

    # Check desktop methods
    print(f'\nDesktop methods:')
    print(f'  open_application is coroutine: {inspect.iscoroutinefunction(d.open_application)}')

asyncio.run(test())