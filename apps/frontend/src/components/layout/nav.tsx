import { Link } from 'react-router';
import Music2 from '~icons/lucide/music-2';
import ChevronRight from '~icons/lucide/chevron-right';
import { Menu } from '@base-ui/react';
import clsx from 'clsx';

const NavMenu = () => {
  return (
    <Menu.Root>
      <Menu.Trigger className="flex items-center justify-center aspect-square cursor-pointer opacity-40 hover:opacity-100 transition-opacity">
        <Music2 />
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Positioner collisionPadding={0}>
          <Menu.Popup
            className={clsx(
              'relative origin-(--transform-origin) bg-pink-50 border-r border-b border-t border-black/10 min-w-32 text-sm',
              'flex flex-col',
              'transition-[scale,opacity]',
              'data-starting-style:scale-[0.98] data-starting-style:opacity-0',
              'data-ending-style:scale-[0.98] data-ending-style:opacity-0',
            )}
          >
            <Menu.Item
              className="px-2 py-1 hover:bg-black/5 transition-colors"
              render={<Link to="/">프로젝트 목록</Link>}
            />
            {/*<Menu.Item
              className="px-2 py-1 hover:bg-black/5 transition-colors"
              render={<Link to="/settings">설정</Link>}
            />*/}
          </Menu.Popup>
        </Menu.Positioner>
      </Menu.Portal>
    </Menu.Root>
  );
};

const NavBreadcrumb = () => {
  return (
    <>
      <Link to="/" className="hover:underline">
        대충프로젝트이름
      </Link>
      <ChevronRight className="size-4 opacity-60" />
      <Link to="/" className="hover:underline">
        #123
      </Link>
    </>
  );
};

const NavAuth = () => {
  return (
    <div className="flex items-center px-3">
      <Link to="/login" className="hover:underline">
        [로그인]
      </Link>
    </div>
  );
};

export const Nav = () => {
  return (
    <nav className="border-b border-black/10 h-10 flex text-sm font-light sticky top-0 bg-pink-50 z-50">
      <NavMenu />
      <div className="flex px-3 items-center gap-2 border-l border-black/10">
        <NavBreadcrumb />
      </div>
      <div className="grow" />
      <NavAuth />
    </nav>
  );
};
