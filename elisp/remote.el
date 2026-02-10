;;; remote.el --- Testing out controlling bevy_m8 remotely -*- lexical-binding: t; -*-
;;
;; Copyright (C) 2026 Luke Holland
;;
;; Author: Luke Holland
;; Maintainer: Luke Holland
;; Created: February 10, 2026
;; Modified: February 10, 2026
;; Version: 0.0.1
;; Keywords: abbrev bib c calendar comm convenience data docs emulations extensions faces files frames games hardware help hypermedia i18n internal languages lisp local maint mail matching mouse multimedia news outlines processes terminals tex text tools unix vc wp
;; Homepage: https://github.com/yelobat/remote
;; Package-Requires: ((emacs "24.3"))
;;
;; This file is not part of GNU Emacs.
;;
;;; Commentary:
;;
;;  Description
;;
;;; Code:

(require 'brpel)

(defalias 'remote-or 'logior
  "Alias of `logior'.")

(defconst remote-edit 1
  "M8 Edit Key Mask Value.")

(defconst remote-option 2
  "M8 Option Key Mask Value.")

(defconst remote-right 4
  "M8 Right Key Mask Value.")

(defconst remote-start 8
  "M8 Start Key Mask Value.")

(defconst remote-select 16
  "M8 Select Key Mask Value.")

(defconst remote-down 32
  "M8 Down Key Mask Value.")

(defconst remote-up 64
  "M8 Up Key Mask Value.")

(defconst remote-left 128
  "M8 Left Key Mask Value.")

(defun remote-trigger-key-press (mask)
  "Triggers the bevy_m8 KeyPress remote event with MASK."
  (brpel-world-trigger-event "bevy_m8::remote::M8Event" `((KeyPress . ,mask))))

(defun remote-trigger-key-hold (mask)
  "Triggers the bevy_m8 KeyHold remote event with MASK."
  (brpel-world-trigger-event "bevy_m8::remote::M8Event" `((KeyHold . ,mask))))

(defun remote-trigger-key-release (mask)
  "Triggers the bevy_m8 KeyRelease remote event with MASK."
  (brpel-world-trigger-event "bevy_m8::remote::M8Event" `((KeyRelease . ,mask))))

(defun remote-do-while-held (mask func)
  "While holding MASK, perform FUNC. Release MASK when done."
  (remote-trigger-key-hold mask)
  (funcall func)
  (remote-trigger-key-release mask))

(defun remote-press-edit ()
  "Send the M8 Edit command remotely, releasing it afterwards."
  (remote-trigger-key-press remote-edit))

(defun remote-press-option ()
  "Send the M8 Option command remotely, releasing it afterwards."
  (remote-trigger-key-press remote-option))

(defun remote-press-delete ()
  "Send the M8 Edit + Option command remotely, releasing them afterwards.
This combination deletes the element under the cursor."
  (remote-trigger-key-press (remote-or remote-edit remote-option)))

(defun remote-press-right ()
  "Send the M8 Right command remotely, releasing it afterwards."
  (remote-trigger-key-press remote-right))

(defun remote-press-start ()
  "Send the M8 Start command remotely, releasing it afterwards."
  (remote-trigger-key-press remote-start))

(defun remote-press-select ()
  "Send the M8 Select command remotely, releasing it afterwards."
  (remote-trigger-key-press remote-select))

(defun remote-press-down ()
  "Send the M8 Down command remotely, releasing it afterwards."
  (remote-trigger-key-press remote-down))

(defun remote-press-up ()
  "Send the M8 Up command remotely, releasing it afterwards."
  (remote-trigger-key-press remote-up))

(defun remote-press-left ()
  "Send the M8 Left command remotely, releasing it afterwards."
  (remote-trigger-key-press remote-left))

(provide 'remote)
;;; remote.el ends here
