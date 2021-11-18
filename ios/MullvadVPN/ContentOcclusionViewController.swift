//
//  ContentOcclusionViewController.swift
//  MullvadVPN
//
//  Created by pronebird on 18/11/2021.
//  Copyright © 2021 Mullvad VPN AB. All rights reserved.
//

import UIKit

class ContentOcclusionViewController: UIViewController {
    override var preferredStatusBarStyle: UIStatusBarStyle {
        return .lightContent
    }
    
    override func viewDidLoad() {
        super.viewDidLoad()

        view.backgroundColor = .primaryColor

        let logoImageView = UIImageView(image: UIImage(named: "LogoIcon"))
        logoImageView.translatesAutoresizingMaskIntoConstraints = false

        let brandNameImage = UIImage(named: "LogoText")!
        let brandNameImageView = UIImageView(image: brandNameImage)
        brandNameImageView.translatesAutoresizingMaskIntoConstraints = false
        brandNameImageView.contentMode = .scaleAspectFit

        let containerView = UIView()
        containerView.translatesAutoresizingMaskIntoConstraints = false

//        containerView.addSubview(logoImageView)
//        containerView.addSubview(brandNameImageView)
        //view.addSubview(containerView)
        view.addSubview(logoImageView)

        let imageSize = brandNameImage.size
        let brandNameAspectRatio = imageSize.width / max(imageSize.height, 1)

        let scaleFactor: CGFloat = 1

        let brandLogoWidth: CGFloat = 44 * scaleFactor
        let brandNameSpacing: CGFloat = 9
        let brandNameHeight: CGFloat = 18 * scaleFactor

        let containerOffsetX: CGFloat = (brandLogoWidth + brandNameSpacing) * -0.5
        let containerOffsetY: CGFloat = -150 // brandLogoWidth * -0.5

        view.addConstraints([
            logoImageView.centerXAnchor.constraint(equalTo: view.centerXAnchor),
            logoImageView.centerYAnchor.constraint(equalTo: view.centerYAnchor),
            logoImageView.widthAnchor.constraint(equalToConstant: 120),
            logoImageView.heightAnchor.constraint(equalTo: logoImageView.widthAnchor, multiplier: 1),
        ])

//        view.addConstraints([
//            containerView.centerXAnchor.constraint(equalTo: view.centerXAnchor, constant: containerOffsetX),
//            containerView.centerYAnchor.constraint(equalTo: view.centerYAnchor, constant: containerOffsetY),
//
//            containerView.leadingAnchor.constraint(greaterThanOrEqualTo: view.leadingAnchor, constant: 8),
//            containerView.trailingAnchor.constraint(greaterThanOrEqualTo: view.trailingAnchor, constant: 8),
//
//            brandNameImageView.centerYAnchor.constraint(equalTo: containerView.centerYAnchor),
//            brandNameImageView.leadingAnchor.constraint(equalTo: logoImageView.trailingAnchor, constant: brandNameSpacing),
//            brandNameImageView.trailingAnchor.constraint(equalTo: containerView.trailingAnchor),
//            brandNameImageView.widthAnchor.constraint(equalTo: brandNameImageView.heightAnchor, multiplier: brandNameAspectRatio),
//            brandNameImageView.heightAnchor.constraint(equalToConstant: brandNameHeight),
//
//            logoImageView.topAnchor.constraint(equalTo: containerView.topAnchor),
//            logoImageView.bottomAnchor.constraint(equalTo: containerView.bottomAnchor),
//            logoImageView.leadingAnchor.constraint(equalTo: containerView.leadingAnchor),
//            logoImageView.widthAnchor.constraint(equalToConstant: brandLogoWidth),
//            logoImageView.heightAnchor.constraint(equalTo: logoImageView.widthAnchor, multiplier: 1),
//        ])
    }
}
