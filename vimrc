" Tab length
set tabstop=4
set shiftwidth=4
" Show line numbers
" set number
" Autoindent Text
set autoindent
" Allow to select the character after the last one in normal mode
set virtualedit=onemore
" Show Position
set ruler
" Highlight syntax
syntax on
" Show tabs
set list listchars=tab:\»\

" mappings
nmap ; :
nmap <BS> <Left>x
imap <Esc><Space> <Esc>

" Normal move around shortcuts

" Alt with Up and down
imap <Esc><Up> <C-O>k
imap <Esc><Down> <C-O>j
nmap <Esc><Up> k
nmap <Esc><Down> j

" Alt with Left and Right
nmap <Esc>f <S-Right>
nmap <Esc>b <S-Left>
imap <Esc>f <S-Right>
imap <Esc>b <S-Left>

" Ctrl-a and Ctrl-e
nmap <C-a> <Home>
nmap <C-e> <End><Right>
imap <C-a> <Home>
imap <C-e> <End>

" Maybe remap the directions like I am used to in all other progs
" nmap i <Up>
" nmap k <Down>
" nmap j <Left>
" nmap l <Right>
" nmap o i

" Too much power in vim for now
nmap h <Nop>
nmap w <Nop>
nmap e <Nop>
nmap r <Nop>
nmap { <Nop>
nmap } <Nop>
nmap b <Nop>
nmap Q <Nop>
nmap K <Nop>
nmap <Enter> <Nop>

" Quit and save
nmap q :wq<Enter>
nmap <C-q> :wq<CR>
" Easy save
nmap s :w<Enter>
" Paste like I expect to
nmap p P
" Standard move down shortcut
nmap <C-v> <C-d>
" Make it work in insert mode
imap <C-v> <C-o><C-d>

" Copy, because I cut with my left hand
vmap y c
" Cut, it is the common cut shortcut
vmap d x

" This bugs the arrow keys for some reason
" inoremap <silent> <Esc> <C-O>:stopinsert<CR>

" Insert mode ease of use
" Save
imap <C-s> <C-o>:w<CR>
" Quit
imap <C-q> <C-o>:wq<CR>
" Delete word
imap <C-w> <C-o>diw
" Undo
imap <C-z> <C-o>u

" Flashing screen is annoying
set visualbell t_vb=
" Show which mode we are in
set noshowmode
" Faster switch to normal mode
" set esckeys
set timeoutlen=10 ttimeoutlen=0

set laststatus=2
" Not useful if powerline is activated
" set statusline+=%t:%m\%l,%c%=%P

" Powerline status bar
python from powerline.vim import setup as powerline_setup
python powerline_setup()
python del powerline_setup

" execute pathogen#infect()
