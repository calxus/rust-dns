% =============================================================================
% l3proj.cls
%
% Template for level 3 projects in the Department of Computing Science at the
% University of Glasgow. Please contact Colin Perkins <csp@dcs.gla.ac.uk> with 
% any questions, comments, or corrections.
%
% =============================================================================
\NeedsTeXFormat{LaTeX2e}
\ProvidesClass{l3proj}[%
2009/01/09 Colin Perkins <csp@dcs.gla.ac.uk>.%
2009/09/16 Modified by Tim Storer <tws@dcs.gla.ac.uk> to support non pdf modes.%
2011/02/23 Modified by Jeremy.Singer@glasgow.ac.uk to use hyperref.%
]
\DeclareOption*{\PassOptionsToClass{\CurrentOption}{report}}
\ProcessOptions
\LoadClass[11pt,a4paper]{report}

\usepackage{ifpdf}
\ifpdf
\usepackage[pdftex]{graphicx}
\else
\usepackage{graphicx}
\fi
\usepackage{times}
\usepackage [colorlinks=true]{hyperref}
\setlength{\parindent}{0pt}
\setlength{\parskip}{11pt plus 2pt}
\addtolength\textwidth{1.0in}
\addtolength\oddsidemargin{-0.50in}
\addtolength\textheight{1.0in}
\addtolength\topmargin{-0.5in}

\renewcommand\maketitle{\begin{titlepage}%
\let\footnotesize\small
\let\footnoterule\relax
\let\footnote \thanks
\includegraphics[trim=0.7cm 0.7cm 0.7cm 0.7cm]{images/CompSci_mono}
  \Large
  \vskip 8em%
  {\LARGE \@title \par}%
  \vskip 3em%
  \large
  \lineskip .75em%
  \begin{tabular}[t]{@{}l}%
    \@author
  \end{tabular}\par%
  \Large
  \vskip 6em%
  Level 3 Project --- \@date 
  \vfill
\end{titlepage}%
\setcounter{footnote}{0}%
\global\let\thanks\relax
\global\let\maketitle\relax
\global\let\@thanks\@empty
\global\let\@author\@empty
\global\let\@date\@empty
\global\let\@title\@empty
\global\let\title\relax
\global\let\author\relax
\global\let\date\relax
\global\let\and\relax
}

\newcommand{\educationalconsent}{
\newpage
\section*{Education Use Consent}

I hereby give my permission for this project to be shown to other
University of Glasgow students and to be distributed in an electronic
format.  \textbf{Please note that you are under no obligation to sign 
this declaration, but doing so would help future students.}

\begin{tabular}{@{}llll}
      &                     &            &                     \\
      &                     &            &                     \\
Name: & \rule{4.0cm}{0.5mm} & Signature: & \rule{4.5cm}{0.5mm} \\
\end{tabular}
}

