import { Link } from 'react-router';
import Music2 from '~icons/lucide/music-2';
import LucideUser from '~icons/lucide/user';
import ChevronRight from '~icons/lucide/chevron-right';
import { Avatar, Menu } from '@base-ui/react';
import { useSuspenseQuery } from '@tanstack/react-query';
import { userQueryOptions } from '../../queries/user';
import { menu } from '../menu';

const NavMenu = () => {
  return (
    <Menu.Root>
      <Menu.Trigger className="flex items-center justify-center aspect-square cursor-pointer opacity-40 hover:opacity-100 transition-opacity">
        <Music2 />
      </Menu.Trigger>
      <Menu.Portal>
        <Menu.Positioner collisionPadding={0}>
          <Menu.Popup className={menu.popup({ className: 'border-l-0' })}>
            <Menu.Item className={menu.item()} render={<Link to="/">프로젝트</Link>} />
            <Menu.Item className={menu.item()} render={<Link to="/teams">팀</Link>} />
            <Menu.Item className={menu.item()} render={<Link to="/agent">에이전트</Link>} />
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
  const { data: user } = useSuspenseQuery(userQueryOptions());

  return (
    <div className="flex items-center px-3">
      {user ? (
        <Menu.Root>
          <Menu.Trigger
            render={
              <Avatar.Root className="size-6 rounded-full overflow-hidden select-none">
                <Avatar.Image src={user.avatar_url} draggable="false" />
                <Avatar.Fallback className="flex justify-center items-center size-full bg-black/5">
                  <LucideUser className="size-4 opacity-40" />
                </Avatar.Fallback>
              </Avatar.Root>
            }
          />
          <Menu.Portal>
            <Menu.Positioner className={menu.positioner()} sideOffset={4}>
              <Menu.Popup className={menu.popup()}>
                <Menu.Item
                  className={menu.item()}
                  onClick={() => {
                    localStorage.removeItem('kanade.apikey');
                    location.reload();
                  }}
                  render={<button />}
                >
                  로그아웃
                </Menu.Item>
                {/*<Menu.Item
                       className="px-2 py-1 hover:bg-black/5 transition-colors"
                       render={<Link to="/settings">설정</Link>}
                     />*/}
              </Menu.Popup>
            </Menu.Positioner>
          </Menu.Portal>
        </Menu.Root>
      ) : (
        <Link to="/login" className="hover:underline">
          [로그인]
        </Link>
      )}
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
